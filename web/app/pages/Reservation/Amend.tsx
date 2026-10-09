import { isAxiosError } from "axios";
import { useCallback, useEffect, useMemo, useState } from "react";
import { redirect, useNavigate } from "react-router";
import { loadTossPayments } from "@tosspayments/tosspayments-sdk";
import type { TossPaymentsWidgets } from "@tosspayments/tosspayments-sdk";

import "./Create.css";
import type { Route } from "./+types/Amend";
import BookingService from "../../api/booking";
import { DEFAULT_UNIT_ID } from "../../constants";
import { useAuth } from "../../context/AuthContext";
import { useEnv } from "../../context/EnvContext";
import { checkSlots, toUtcIso8601 } from "../../lib/datetime";
import { defaultErrorHandler } from "../../lib/error";
import RequiresAuth from "../../lib/RequiresAuth";
import AttachmentSelection from "../../components/AttachmentSelection";
import Calendar from "../../components/Calendar";
import Section from "../../components/Section";
import type { BookingAttachmentId, UserId } from "../../types/models/base";
import type {
  Booking,
  BookingAttachment,
  OccupiedSlot,
} from "../../types/models/booking";

export function meta(): Route.MetaDescriptors {
  return [{ title: "예약 변경 | 드림하우스 합주실" }];
}

interface LoaderData {
  booking: Booking;
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.bookingId) {
    throw new Error("bookingId is not supplied");
  }

  try {
    const result = await BookingService.get(params.bookingId);

    return {
      booking: result.data.booking,
    };
  } catch (error) {
    defaultErrorHandler(error);
    throw redirect(`/reservation/${params.bookingId}`);
  }
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
  currentHours: number;
  slots: OccupiedSlot[];

  selectedHours: number | null;
  onSelectHours: (hours: number | null) => void;
}

function HoursPicker(props: HoursPickerProps) {
  const { currentHours, selectedTime, maxBookingHours, slots } = props;

  const items = [];

  for (let i = 1; i <= maxBookingHours; ++i) {
    const date = new Date(selectedTime.getTime() + (i - 1) * 60 * 60 * 1000);
    if (checkSlots(date, slots)) {
      break;
    }

    const disabled = currentHours > i;

    items.push(
      <span className="time-selection" key={i}>
        <input
          type="radio"
          name="hours"
          checked={props.selectedHours === i}
          onChange={() => props.onSelectHours(i)}
          disabled={disabled}
          id={`hours-${i}`}
        />
        <label htmlFor={`hours-${i}`}>{i}시간</label>
      </span>,
    );
  }

  return <div className="time-picker">{items}</div>;
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

function AmendReservation({ loaderData }: Route.ComponentProps) {
  const { booking } = loaderData;

  const auth = useAuth();
  const navigate = useNavigate();

  const [start, setStart] = useState(new Date());
  const [end, setEnd] = useState(new Date());
  const [maxBookingHours, setMaxBookingHours] = useState(1);
  const [slots, setSlots] = useState<OccupiedSlot[]>([]);
  const [now, setNow] = useState(new Date());
  const [errorMessage, setErrorMessage] = useState("");

  const [selectedDate, setSelectedDate] = useState<Date | null>(null);
  const [selectedTime, setSelectedTime] = useState<Date | null>(null);
  const [selectedHours, setSelectedHours] = useState<number | null>(
    booking.bookingHours,
  );
  const [price, setPrice] = useState<number | null>(null);

  const [attachments, setAttachments] = useState<BookingAttachment[]>([]);
  const [unavailableAttachmentIds, setUnavailableAttachmentIds] = useState<
    BookingAttachmentId[]
  >([]);
  const [selectedAttachmentIds, setSelectedAttachmentIds] = useState<
    BookingAttachmentId[]
  >([]);
  const includedAttachmentIds = useMemo(
    () => booking.attachments.map((v) => v.id),
    [booking],
  );

  const [tossPaymentsWidgets, setTossPaymentsWidgets] =
    useState<TossPaymentsWidgets | null>(null);
  const [hasAgreedRequiredTerms, setAgreedRequiredTerms] = useState(true);

  const [isRequestInProgress, setRequestInProgress] = useState(false);

  // Time is left unchanged unless a new time range is fully selected, so that
  // attachments alone can be added to the booking
  const isTimeSelected = selectedTime !== null && selectedHours !== null;
  const isTimeSelectionIncomplete = selectedDate !== null && !isTimeSelected;

  const isAvailable =
    !isTimeSelectionIncomplete &&
    (isTimeSelected || selectedAttachmentIds.length > 0) &&
    !isRequestInProgress &&
    hasAgreedRequiredTerms;

  const proceed = async () => {
    if (isTimeSelectionIncomplete) {
      return;
    }

    const additionalHours =
      selectedHours !== null ? selectedHours - booking.bookingHours : 0;

    setRequestInProgress(true);
    try {
      const amendResult = await BookingService.amend(booking.id, {
        newTimeFrom:
          selectedTime !== null ? toUtcIso8601(selectedTime) : undefined,
        additionalHours,
        attachmentIds: selectedAttachmentIds,
      });

      if (amendResult.data.foreignPaymentId) {
        const orderNames = [];
        if (additionalHours > 0) {
          orderNames.push(`추가 공간이용료 (${additionalHours} 시간)`);
        }
        if (selectedAttachmentIds.length > 0) {
          orderNames.push(`대여물품 (${selectedAttachmentIds.length}개)`);
        }

        await tossPaymentsWidgets?.requestPayment({
          orderId: amendResult.data.foreignPaymentId,
          orderName: orderNames.join(", "),
          successUrl:
            window.location.origin + "/reservation/payment/toss/success/",
          failUrl: window.location.origin + "/reservation/payment/toss/fail/",
        });
      } else {
        alert("예약이 변경되었습니다.");
        navigate(`/reservation/${booking.id}`);
      }
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
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

  const fetchCalendar = useCallback(async () => {
    try {
      const result = await BookingService.calendar(DEFAULT_UNIT_ID, booking.id);
      setStart(new Date(result.data.start));
      setEnd(new Date(result.data.end));
      setMaxBookingHours(result.data.maxBookingHours);
      setSlots(result.data.slots);
    } catch (error) {
      defaultErrorHandler(error);
    }
  }, [booking]);

  useEffect(() => {
    fetchCalendar();

    window.addEventListener("focus", fetchCalendar);

    return () => {
      window.removeEventListener("focus", fetchCalendar);
    };
  }, [fetchCalendar]);

  useEffect(() => {
    setInterval(() => {
      setNow(new Date());
    }, 10000);
  });

  useEffect(() => {
    let isStale = false;

    const fetchAttachments = async () => {
      try {
        const result =
          selectedTime !== null && selectedHours !== null
            ? await BookingService.getAttachments(
                selectedTime,
                selectedHours,
                booking.id,
              )
            : await BookingService.getAttachments(
                new Date(booking.bookingStart),
                booking.bookingHours,
                booking.id,
              );
        if (isStale) {
          return;
        }

        const unavailableIds = result.data.unavailableAttachmentIds;
        setAttachments(result.data.attachments);
        setUnavailableAttachmentIds(unavailableIds);
        setSelectedAttachmentIds((ids) =>
          ids.filter((id) => !unavailableIds.includes(id)),
        );
      } catch (error) {
        defaultErrorHandler(error);
      }
    };

    fetchAttachments();

    return () => {
      isStale = true;
    };
  }, [booking, selectedTime, selectedHours]);

  useEffect(() => {
    const checkAvailability = async () => {
      const isTimeSelected = selectedTime !== null && selectedHours !== null;
      if (!isTimeSelected && selectedAttachmentIds.length === 0) {
        setPrice(null);
        return;
      }

      setErrorMessage("");

      const timeFrom = selectedTime ?? new Date(booking.bookingStart);
      const desiredHours = selectedHours ?? booking.bookingHours;

      try {
        const result = await BookingService.check({
          unitId: DEFAULT_UNIT_ID,
          desiredHours,
          timeFrom: toUtcIso8601(timeFrom),
          additionalHours: desiredHours - booking.bookingHours,
          excludeBookingId: booking.id,
          attachmentIds: [...includedAttachmentIds, ...selectedAttachmentIds],
        });
        if (result.data.totalPrice > 0) {
          setPrice(result.data.totalPrice);
        }
      } catch (error) {
        if (isAxiosError(error)) {
          if (error.response?.data.message) {
            setErrorMessage(error.response?.data.message ?? "");
          }
        }
      }
    };

    checkAvailability();
  }, [
    booking,
    selectedTime,
    selectedHours,
    includedAttachmentIds,
    selectedAttachmentIds,
  ]);

  return (
    <>
      <Section id="booking-info" title="현재 예약 시간">
        <span>
          {new Date(booking.bookingStart).toLocaleString()} -{" "}
          {new Date(booking.bookingEnd).toLocaleString()} (총{" "}
          {booking.bookingHours} 시간)
        </span>
      </Section>
      <Section id="calendar" title="변경 일시 선택">
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
            currentHours={booking.bookingHours}
            slots={slots}
            selectedHours={selectedHours}
            onSelectHours={setSelectedHours}
          />
        ) : null}
        {errorMessage ? <p className="error-message">{errorMessage}</p> : null}
      </Section>
      {attachments.length > 0 ? (
        <Section id="attachments" title="대여물품 추가">
          <AttachmentSelection
            attachments={attachments}
            unavailableAttachmentIds={unavailableAttachmentIds}
            includedAttachmentIds={includedAttachmentIds}
            selectedAttachmentIds={selectedAttachmentIds}
            onSelectAttachmentIds={setSelectedAttachmentIds}
          />
        </Section>
      ) : null}
      {auth && price !== null && price > 0 ? (
        <Section id="payment" title="결제 정보">
          <TossPayment
            userId={auth.user.id}
            price={price}
            tossPaymentsWidgets={tossPaymentsWidgets}
            setTossPaymentsWidgets={setTossPaymentsWidgets}
            setAgreedRequiredTerms={setAgreedRequiredTerms}
          />
        </Section>
      ) : null}
      <div className="create-actions">
        <button className="cta" disabled={!isAvailable} onClick={proceed}>
          예약 변경
        </button>
      </div>
    </>
  );
}

export default RequiresAuth(AmendReservation);
