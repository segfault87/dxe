import { useState } from "react";
import { useNavigate } from "react-router";

import type { Route } from "./+types/AdhocReservations";
import AdminService from "../../api/admin";
import { DEFAULT_UNIT_ID } from "../../constants";
import { toUtcIso8601 } from "../../lib/datetime";
import { defaultErrorHandler, handleUnauthorizedError } from "../../lib/error";
import type { AdhocReservationId } from "../../types/models/base";
import type { AdhocReservation } from "../../types/models/booking";

interface LoaderData {
  reservations: AdhocReservation[];
}

export async function clientLoader({}: Route.ClientActionArgs): Promise<LoaderData> {
  try {
    const result = await AdminService.getAdhocReservations(DEFAULT_UNIT_ID);

    return {
      reservations: result.data.reservations,
    };
  } catch (error) {
    handleUnauthorizedError(error);

    return { reservations: [] };
  }
}

export default function AdhocReservations({
  loaderData,
}: Route.ComponentProps) {
  const { reservations } = loaderData;
  const navigate = useNavigate();

  const deleteReservation = async (reservationId: AdhocReservationId) => {
    if (!confirm("정말로 삭제하시겠습니까?")) {
      return;
    }

    try {
      await AdminService.deleteAdhocReservation(reservationId);
      navigate(0);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  const [startTime, setStartTime] = useState(toUtcIso8601(new Date()));
  const [identityId, setIdentityId] = useState("");
  const [desiredHours, setDesiredHours] = useState("2");
  const [expiresAt, setExpiresAt] = useState("");
  const [remark, setRemark] = useState("");

  const createReservation = async () => {
    try {
      await AdminService.createAdhocReservation({
        unitId: DEFAULT_UNIT_ID,
        customerId: identityId,
        timeFrom: startTime,
        desiredHours: parseInt(desiredHours),
        expiresAt: expiresAt ? expiresAt : null,
        remark: remark.length === 0 ? null : remark,
      });

      navigate(0);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  return (
    <>
      <h2>임의 예약 목록</h2>
      <table>
        <tr>
          <th>고객명</th>
          <th>시작시간</th>
          <th>종료시간</th>
          <th>임시</th>
          <th>비고</th>
          <th>동작</th>
        </tr>
        {reservations.map((v) => (
          <tr key={v.id}>
            <td>{v.holder.name}</td>
            <td>{new Date(v.reservationStart).toLocaleString()}</td>
            <td>{new Date(v.reservationEnd).toLocaleString()}</td>
            <td>
              {v.deletedAt ? new Date(v.deletedAt).toLocaleString() : "-"}
            </td>
            <td>{v.remark ?? ""}</td>
            <td>
              <button
                className="primary"
                onClick={() => deleteReservation(v.id)}
              >
                삭제
              </button>
            </td>
          </tr>
        ))}
      </table>
      <h2>새 예약 생성</h2>
      시작시간:{" "}
      <input
        type="text"
        value={startTime}
        onChange={(e) => {
          setStartTime(e.target.value);
        }}
      />{" "}
      이용시간:{" "}
      <input
        type="text"
        value={desiredHours}
        onChange={(e) => {
          setDesiredHours(e.target.value);
        }}
      />
      <br />
      비고:{" "}
      <input
        type="text"
        value={remark}
        onChange={(e) => {
          setRemark(e.target.value);
        }}
      />
      <br />
      고객 ID:{" "}
      <input
        type="text"
        value={identityId}
        onChange={(e) => {
          setIdentityId(e.target.value);
        }}
      />
      <br />
      만료시간:{" "}
      <input
        type="text"
        value={expiresAt}
        onChange={(e) => {
          setExpiresAt(e.target.value);
        }}
      />
      <br />
      <button onClick={createReservation}>생성</button>
    </>
  );
}
