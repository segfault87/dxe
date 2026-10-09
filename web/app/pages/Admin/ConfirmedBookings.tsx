import { useNavigate } from "react-router";
import type { NavigateFunction } from "react-router";

import type { Route } from "./+types/ConfirmedBookings";
import AdminService from "../../api/admin";
import { loaderErrorHandler } from "../../lib/error";
import type { BookingId } from "../../types/models/base";
import type { BookingWithPayments } from "../../types/models/booking";

interface LoaderData {
  bookings: BookingWithPayments[];
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const result = await AdminService.getBookings("confirmed", new Date());

    return {
      bookings: result.data.bookings,
    };
  } catch (error) {
    throw loaderErrorHandler(error, request.url);
  }
}

async function cancelBooking(id: BookingId, navigate: NavigateFunction) {
  const result = await AdminService.cancelBooking(id);

  if (result.status === 200) {
    if (result.data.transaction) {
      alert(`성공했습니다. 환불액: ${result.data.transaction.tossPayments?.refundPrice}`);
      navigate(0);
    }
  } else {
    alert(`오류가 발생했습니다: ${result.statusText}`)
  }
}

export default function PendingBookings({ loaderData }: Route.ComponentProps) {
  const { bookings } = loaderData;
  const navigate = useNavigate();

  return (
    <>
      <h2>확정 예약 목록</h2>
      <table>
        <tr>
          <th>ID</th>
          <th>고객명</th>
          <th>예약자명</th>
          <th>시작시간</th>
          <th>종료시간</th>
          <th>생성시각</th>
          <th>확정 시각</th>
          <th>대여물품</th>
          <th>동작</th>
        </tr>
        {bookings.map((e) => (
          <tr key={e.booking.id}>
            <td>{e.booking.id}</td>
            <td>{e.booking.customer.name}</td>
            <td>{e.booking.holder.name}</td>
            <td>{new Date(e.booking.bookingStart).toLocaleString()}</td>
            <td>{new Date(e.booking.bookingEnd).toLocaleString()}</td>
            <td>{new Date(e.booking.createdAt).toLocaleString()}</td>
            <td>{new Date(e.booking.confirmedAt!).toLocaleString()}</td>
            <td>
              {e.booking.attachments ? (
                <ul>
                  {e.booking.attachments.map((e) => <li>{e.name}</li>)}
                </ul>
              ) : null}
            </td>
            <td><button onClick={() => cancelBooking(e.booking.id, navigate)}>환불</button></td>
          </tr>
        ))}
      </table>
    </>
  );
}
