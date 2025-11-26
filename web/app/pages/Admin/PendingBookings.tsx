import { useNavigate } from "react-router";

import type { Route } from "./+types/PendingBookings";
import AdminService from "../../api/admin";
import { defaultErrorHandler } from "../../lib/error";
import type { BookingId } from "../../types/models/base";
import type { BookingWithPayments } from "../../types/models/booking";
import type { BookingAction } from "../../types/handlers/admin";

interface LoaderData {
  bookings: BookingWithPayments[];
}

export async function clientLoader({}: Route.ClientLoaderArgs): Promise<LoaderData> {
  const result = await AdminService.getBookings("pending");

  return {
    bookings: result.data.bookings,
  };
}

export default function PendingBookings({ loaderData }: Route.ComponentProps) {
  const { bookings } = loaderData;
  const navigate = useNavigate();

  const modifyBooking = async (bookingId: BookingId, action: BookingAction) => {
    try {
      await AdminService.modifyBooking(bookingId, {
        action,
      });

      navigate(0);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  return (
    <>
      <h2>대기 예약 목록</h2>
      <table>
        <tr>
          <th>고객명</th>
          <th>시작시간</th>
          <th>종료시간</th>
          <th>상태</th>
          <th>동작</th>
        </tr>
        {bookings.map((e) => (
          <tr key={e.booking.id}>
            <td>{e.booking.customer.name}</td>
            <td>{new Date(e.booking.bookingStart).toLocaleString()}</td>
            <td>{new Date(e.booking.bookingEnd).toLocaleString()}</td>
            <td>{e.booking.status}</td>
            <td>
              {e.booking.status === "PENDING" ? (
                <button onClick={() => modifyBooking(e.booking.id, "CONFIRM")}>
                  확정
                </button>
              ) : null}
              <button onClick={() => modifyBooking(e.booking.id, "CANCEL")}>
                취소
              </button>
            </td>
          </tr>
        ))}
      </table>
    </>
  );
}
