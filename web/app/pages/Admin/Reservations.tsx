import { useState } from "react";
import { useNavigate } from "react-router";

import type { Route } from "./+types/Reservations";
import AdminService from "../../api/admin";
import { DEFAULT_UNIT_ID } from "../../constants";
import { toUtcIso8601 } from "../../lib/datetime";
import { defaultErrorHandler } from "../../lib/error";
import type { ReservationId } from "../../types/models/base";
import type { Reservation } from "../../types/models/booking";

interface LoaderData {
  reservations: Reservation[];
}

export async function clientLoader({}: Route.ClientActionArgs): Promise<LoaderData> {
  const result = await AdminService.getReservations(DEFAULT_UNIT_ID);

  return {
    reservations: result.data.reservations,
  };
}

export default function Reservations({ loaderData }: Route.ComponentProps) {
  const { reservations } = loaderData;
  const navigate = useNavigate();

  const deleteReservation = async (reservationId: ReservationId) => {
    try {
      await AdminService.deleteReservation(reservationId);
      navigate(0);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  const [startTime, setStartTime] = useState(toUtcIso8601(new Date()));
  const [desiredHours, setDesiredHours] = useState("2");
  const [isTemporary, setTemporary] = useState(false);
  const [remark, setRemark] = useState("");

  const createReservation = async () => {
    try {
      await AdminService.createReservation({
        unitId: DEFAULT_UNIT_ID,
        timeFrom: startTime,
        desiredHours: parseInt(desiredHours),
        temporary: isTemporary,
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
            <td>{v.temporary}</td>
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
      <input
        type="checkbox"
        checked={isTemporary}
        onChange={(e) => setTemporary(e.target.checked)}
        id="is-temporary"
      />{" "}
      <label htmlFor="is-temporary">임시</label>
      <br />
      <button onClick={createReservation}>생성</button>
    </>
  );
}
