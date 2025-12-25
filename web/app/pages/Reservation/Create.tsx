import { isAxiosError } from "axios";
import { useCallback, useEffect, useState } from "react";
import ReactGA from "react-ga4";
import { useNavigate } from "react-router";
import { loadTossPayments } from "@tosspayments/tosspayments-sdk";
import type { TossPaymentsWidgets } from "@tosspayments/tosspayments-sdk";

import "./Create.css";
import type { Route } from "./+types/Create";
import BookingService from "../../api/booking";
import UserService from "../../api/user";
import { DEFAULT_UNIT_ID } from "../../constants";
import type { AuthContextData } from "../../context/AuthContext";
import { useEnv } from "../../context/EnvContext";
import { checkSlots, toUtcIso8601 } from "../../lib/datetime";
import { defaultErrorHandler } from "../../lib/error";
import RequiresAuth, { type AuthProps } from "../../lib/RequiresAuth";
import Calendar from "../../components/Calendar";
import GroupInvitationModal from "../../components/GroupInvitation";
import Section from "../../components/Section";
import type {
  AdhocReservationId,
  IdentityId,
  UserId,
} from "../../types/models/base";
import type { OccupiedSlot } from "../../types/models/booking";
import type { GroupWithUsers } from "../../types/models/group";
import ReservationCalendar from "../../components/ReservationCalendar";

export function meta(): Route.MetaDescriptors {
  return [{ title: "예약 | 드림하우스 합주실" }];
}

interface TimePickerProps {
  selectedDate: Date;
  now: Date;
  slots: OccupiedSlot[];

  selectedTime: Date | null;
  onSelectTime: (date: Date | null) => void;
}

function TimePicker(props: TimePickerProps) {
  const { selectedDate, now, slots } = props;

  const items = [];

  for (let i = 0; i < 24; ++i) {
    const date = new Date(selectedDate.getTime() + i * 60 * 60 * 1000);
    const isDisabled = date < now || checkSlots(date, slots);

    const hours = date.getHours().toString().padStart(2, "0");
    items.push(
      <span className="time-selection" key={i}>
        <input
          type="radio"
          name="time"
          checked={props.selectedTime?.getTime() === date.getTime()}
          onChange={() => props.onSelectTime(date)}
          id={`time-${i}`}
          disabled={isDisabled}
        />
        <label htmlFor={`time-${i}`}>{hours}:00</label>
      </span>,
    );
  }

  return <div className="time-picker">{items}</div>;
}

interface HoursPickerProps {
  selectedTime: Date;
  maxBookingHours: number;
  slots: OccupiedSlot[];

  selectedHours: number | null;
  onSelectHours: (hours: number | null) => void;
}

function HoursPicker(props: HoursPickerProps) {
  const { selectedTime, maxBookingHours, slots } = props;

  const items = [];

  for (let i = 1; i <= maxBookingHours; ++i) {
    const date = new Date(selectedTime.getTime() + (i - 1) * 60 * 60 * 1000);
    if (checkSlots(date, slots)) {
      break;
    }

    items.push(
      <span className="time-selection" key={i}>
        <input
          type="radio"
          name="hours"
          checked={props.selectedHours === i}
          onChange={() => props.onSelectHours(i)}
          id={`hours-${i}`}
        />
        <label htmlFor={`hours-${i}`}>{i}시간</label>
      </span>,
    );
  }

  return <div className="time-picker">{items}</div>;
}

interface CustomerSelectionProps {
  auth: AuthContextData;
  selectedIdentityId: IdentityId | null;
  onSelectIdentityId: (identityId: IdentityId) => void;
}

interface GroupWithCreationTag extends GroupWithUsers {
  newlyCreated?: boolean;
}

function CustomerSelection({
  auth,
  selectedIdentityId,
  onSelectIdentityId,
}: CustomerSelectionProps) {
  const user = auth.user;

  const [groups, setGroups] = useState<GroupWithCreationTag[] | null>(null);
  const [groupInvitationModal, setGroupInvitationModal] =
    useState<GroupWithUsers | null>(null);

  const [newGroupName, setNewGroupName] = useState("");
  const [isRequestInProgress, setRequestInProgress] = useState(false);

  useEffect(() => {
    const fetchGroups = async () => {
      try {
        const result = await UserService.listGroups();
        setGroups(result.data.groups);
      } catch (error) {
        defaultErrorHandler(error);
      }
    };

    fetchGroups();
  }, [user]);

  if (!user || groups === null) {
    return null;
  }

  const createGroup = async (name: string) => {
    try {
      setRequestInProgress(true);
      const result = await UserService.createGroup({ name });
      const group = {
        newlyCreated: true,
        ...result.data.group,
      };
      setGroups([...groups, group]);
      setNewGroupName("");
      onSelectIdentityId(group.id);
      ReactGA.event("create_new_group", { from: "reservation" });
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const groupGuide =
    groups.length === 0 ? (
      <>
        <h4>단체 이용객이신가요?</h4>
        <p>
          단체로 이용하실 경우 그룹을 만들고 그룹 명의로 예약하시는 것을
          권장합니다. 그룹 명의로 예약 시 다음 장점들이 있습니다.
        </p>
        <ul>
          <li>
            그룹에 소속된 구성원들 모두 예약 정보를 실시간으로 확인할 수
            있습니다.
          </li>
          <li>
            예약 확정시 구성원 전부에게 카카오톡으로 이용 안내 링크를 자동으로
            보내드립니다.
          </li>
          <li>
            구성원들이 모두 도어록을 잠금해제할 수 있습니다. (개인 명의로 예약
            시 예약자 본인만 개방이 가능합니다)
          </li>
          <li>차량번호를 입력한 구성원들 모두 무료주차가 적용됩니다.</li>
        </ul>
        <p>그룹을 만드시려면 아래 입력란에 그룹명을 입력한 후 추가해 주세요.</p>
      </>
    ) : null;

  return (
    <div className="customer-selection">
      <ul className="identities">
        <li key={user.id}>
          <input
            type="radio"
            name="identityId"
            checked={selectedIdentityId === user.id}
            value={user.id}
            onChange={(e) => onSelectIdentityId(e.target.value)}
            id={`identity-${user.id}`}
          />
          <label htmlFor={`identity-${user.id}`}>
            예약자 본인 ({user.name})
          </label>
        </li>
        {groups.map((group) => {
          return (
            <li key={group.id}>
              <input
                type="radio"
                name="identityId"
                checked={selectedIdentityId === group.id}
                value={group.id}
                onChange={(e) => onSelectIdentityId(e.target.value)}
                id={`identity-${group.id}`}
              />
              <label htmlFor={`identity-${group.id}`}>{group.name}</label>{" "}
              {group.newlyCreated ? (
                <a
                  onClick={() => {
                    setGroupInvitationModal(group);
                  }}
                >
                  구성원 초대
                </a>
              ) : null}
            </li>
          );
        })}
      </ul>
      {groupGuide}
      <hr />
      <input
        className="group-name"
        type="text"
        value={newGroupName}
        placeholder="그룹 추가"
        disabled={isRequestInProgress}
        maxLength={40}
        onChange={(e) => setNewGroupName(e.target.value)}
      />
      <button
        style={{ marginLeft: "8px" }}
        className="primary"
        onClick={() => {
          createGroup(newGroupName);
        }}
        disabled={newGroupName.length === 0 || isRequestInProgress}
      >
        추가
      </button>
      {groupInvitationModal !== null ? (
        <GroupInvitationModal
          isOpen={groupInvitationModal !== null}
          close={() => {
            setGroupInvitationModal(null);
          }}
          group={groupInvitationModal}
        />
      ) : null}
    </div>
  );
}

function CashPayment({
  depositor,
  setDepositor,
  isRequestInProgress,
  price,
}: {
  depositor: string;
  setDepositor: (name: string) => void;
  isRequestInProgress: boolean;
  price: number | null;
}) {
  return (
    <>
      현재는 계좌입금을 통한 결제만 지원합니다. 아래 계좌로 입금해주시면 확인 후
      예약이 확정됩니다.
      <p>
        입금 금액: {price !== null ? <em>₩{price.toLocaleString()}</em> : "-"}
        <br />
        입금 계좌: 신한은행 110-609-686081 (박준규, 디엑스이 스튜디오)
      </p>
      <p>
        <input
          type="text"
          value={depositor}
          placeholder="입금자명 입력"
          disabled={isRequestInProgress}
          maxLength={40}
          onChange={(e) => setDepositor(e.target.value)}
        />
      </p>
    </>
  );
}

function TossPayment({
  userId,
  price,
  tossPaymentsWidgets,
  setTossPaymentsWidgets,
  setAgreedRequiredTerms,
}: {
  userId: UserId;
  price: number | null;
  tossPaymentsWidgets: TossPaymentsWidgets | null;
  setTossPaymentsWidgets: (widgets: TossPaymentsWidgets) => void;
  setAgreedRequiredTerms: (agreed: boolean) => void;
}) {
  const { tossPaymentClientKey } = useEnv();

  const initializeTossPayments = useCallback(async () => {
    const tossPaymentsSdk = await loadTossPayments(tossPaymentClientKey);

    const customerKey = userId;
    const widgets = tossPaymentsSdk.widgets({
      customerKey,
    });

    await widgets.setAmount({
      currency: "KRW",
      value: 0,
    });

    try {
      await widgets.renderPaymentMethods({
        selector: "#toss-payment-method",
        variantKey: "DEFAULT",
      });
      const agreementWidget = await widgets.renderAgreement({
        selector: "#toss-payment-agreement",
        variantKey: "AGREEMENT",
      });

      agreementWidget.on("agreementStatusChange", (agreementStatus) => {
        setAgreedRequiredTerms(agreementStatus.agreedRequiredTerms);
      });

      setTossPaymentsWidgets(widgets);
    } catch {
      // suppress error
    }
  }, [
    setAgreedRequiredTerms,
    setTossPaymentsWidgets,
    userId,
    tossPaymentClientKey,
  ]);

  useEffect(() => {
    initializeTossPayments();
  }, [initializeTossPayments]);

  useEffect(() => {
    if (tossPaymentsWidgets !== null && price !== null) {
      tossPaymentsWidgets.setAmount({ currency: "KRW", value: price });
    }
  }, [tossPaymentsWidgets, price]);

  return (
    <>
      결제 금액: {price !== null ? <em>₩{price.toLocaleString()}</em> : "-"}
      <div id="toss-payment-method" />
      <div id="toss-payment-agreement" />
    </>
  );
}

function Reservation({ auth }: AuthProps) {
  const env = useEnv();

  const [start, setStart] = useState(new Date());
  const [end, setEnd] = useState(new Date());
  const [maxBookingHours, setMaxBookingHours] = useState(1);
  const [slots, setSlots] = useState<OccupiedSlot[]>([]);
  const [now, setNow] = useState(new Date());
  const [errorMessage, setErrorMessage] = useState("");
  const [temporaryReservationId, setTemporaryReservationId] =
    useState<AdhocReservationId | null>(null);

  const [selectedDate, setSelectedDate] = useState<Date | null>(null);
  const [selectedTime, setSelectedTime] = useState<Date | null>(null);
  const [selectedHours, setSelectedHours] = useState<number | null>(null);
  const [price, setPrice] = useState<number | null>(null);
  const [depositor, setDepositor] = useState("");

  const [tossPaymentsWidgets, setTossPaymentsWidgets] =
    useState<TossPaymentsWidgets | null>(null);
  const [hasAgreedRequiredTerms, setAgreedRequiredTerms] = useState(true);

  useEffect(() => {
    if (auth) {
      setDepositor(auth.user.depositorName ?? "");
    }
  }, [auth]);

  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const navigate = useNavigate();

  let isAvailable =
    selectedDate !== null &&
    selectedTime !== null &&
    selectedHours !== null &&
    !isRequestInProgress;

  if (env.enableTossPayments) {
    isAvailable &&= hasAgreedRequiredTerms;
  } else {
    isAvailable &&= depositor.trim().length > 0;
  }

  const [selectedIdentityId, setSelectedIdentityId] =
    useState<IdentityId | null>(null);

  const fetchCalendar = async () => {
    try {
      const result = await BookingService.calendar(DEFAULT_UNIT_ID);
      setStart(new Date(result.data.start));
      setEnd(new Date(result.data.end));
      setMaxBookingHours(result.data.maxBookingHours);
      setSlots(result.data.slots);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  const makeReservation = async () => {
    setRequestInProgress(true);

    if (
      selectedTime === null ||
      selectedHours === null ||
      selectedIdentityId === null ||
      depositor.length === 0
    ) {
      return;
    }

    try {
      const result = await BookingService.submitBooking({
        unitId: DEFAULT_UNIT_ID,
        timeFrom: toUtcIso8601(selectedTime),
        desiredHours: selectedHours,
        identityId: selectedIdentityId,
        depositorName: depositor,
      });

      navigate(`/reservation/${result.data.booking.id}`);
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const proceedTossPayment = async (widgets: TossPaymentsWidgets) => {
    if (
      selectedTime === null ||
      selectedHours === null ||
      selectedIdentityId === null
    ) {
      return;
    }

    setRequestInProgress(true);
    let activeAdhocReservationId: AdhocReservationId | null = null;
    try {
      const initiateResult = await BookingService.initiateTossPayment({
        unitId: DEFAULT_UNIT_ID,
        timeFrom: toUtcIso8601(selectedTime),
        desiredHours: selectedHours,
        identityId: selectedIdentityId,
        temporaryReservationId: temporaryReservationId,
      });
      const data = initiateResult.data;
      activeAdhocReservationId = data.temporaryReservationId;
      setTemporaryReservationId(data.temporaryReservationId);

      await widgets.requestPayment({
        orderId: data.orderId,
        orderName: `공간이용료 (${selectedHours} 시간)`,
        successUrl:
          window.location.origin + "/reservation/payment/toss/success/",
        failUrl: window.location.origin + "/reservation/payment/toss/fail/",
      });
    } catch (error) {
      if (activeAdhocReservationId) {
        await BookingService.deleteTemporaryReservation(
          activeAdhocReservationId,
        );
        setTemporaryReservationId(null);
      }
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const proceedPayment = async () => {
    ReactGA.event("payment_initiate");

    if (env.enableTossPayments && tossPaymentsWidgets !== null) {
      await proceedTossPayment(tossPaymentsWidgets);
    } else if (!env.enableTossPayments) {
      await makeReservation();
    }
  };

  useEffect(() => {
    setSelectedTime(null);
    setPrice(null);
  }, [selectedDate]);
  useEffect(() => {
    setSelectedHours(null);
    setPrice(null);
  }, [selectedTime]);

  useEffect(() => {
    fetchCalendar();

    window.addEventListener("focus", fetchCalendar);

    return () => {
      window.removeEventListener("focus", fetchCalendar);
    };
  }, []);

  useEffect(() => {
    setInterval(() => {
      setNow(new Date());
    }, 10000);
  });

  useEffect(() => {
    const checkAvailability = async () => {
      if (selectedTime === null || selectedHours === null) {
        setPrice(null);
        return;
      }

      setErrorMessage("");

      try {
        const result = await BookingService.check({
          unitId: DEFAULT_UNIT_ID,
          desiredHours: selectedHours,
          timeFrom: toUtcIso8601(selectedTime),
        });
        setPrice(result.data.totalPrice);
      } catch (error) {
        if (isAxiosError(error)) {
          if (error.response?.data.message) {
            setErrorMessage(error.response?.data.message ?? "");
          }
        }
      }
    };

    checkAvailability();
  }, [selectedTime, selectedHours]);

  return (
    <>
      <Section id="calendar" title="예약 현황">
        <div className="reservation-calendar">
          <ReservationCalendar />
        </div>
      </Section>
      <Section id="time-selection" title="일시 선택">
        <Calendar start={start} end={end} onSelect={setSelectedDate} />
        {selectedDate ? (
          <TimePicker
            selectedDate={selectedDate}
            slots={slots}
            now={now}
            selectedTime={selectedTime}
            onSelectTime={setSelectedTime}
          />
        ) : null}
        {selectedTime ? (
          <HoursPicker
            selectedTime={selectedTime}
            maxBookingHours={maxBookingHours}
            slots={slots}
            selectedHours={selectedHours}
            onSelectHours={setSelectedHours}
          />
        ) : null}
        {errorMessage ? <p className="error-message">{errorMessage}</p> : null}
      </Section>
      <Section id="customer" title="예약자 선택">
        <CustomerSelection
          auth={auth}
          selectedIdentityId={selectedIdentityId}
          onSelectIdentityId={setSelectedIdentityId}
        />
      </Section>
      <Section id="payment" title="결제 정보">
        {auth ? (
          env.enableTossPayments ? (
            <TossPayment
              userId={auth.user.id}
              price={price}
              tossPaymentsWidgets={tossPaymentsWidgets}
              setTossPaymentsWidgets={setTossPaymentsWidgets}
              setAgreedRequiredTerms={setAgreedRequiredTerms}
            />
          ) : (
            <CashPayment
              depositor={depositor}
              setDepositor={setDepositor}
              isRequestInProgress={isRequestInProgress}
              price={price}
            />
          )
        ) : null}
      </Section>
      <div className="create-actions">
        <button
          className="cta"
          disabled={!isAvailable}
          onClick={proceedPayment}
        >
          예약하기
        </button>
      </div>
    </>
  );
}

export default RequiresAuth(Reservation, "/reservation/login/");
