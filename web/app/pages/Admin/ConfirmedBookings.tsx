import type { Route } from "./+types/ConfirmedBookings";
import AdminService from "../../api/admin";
import { handleUnauthorizedError } from "../../lib/error";
import type { BookingWithPayments } from "../../types/models/booking";

interface LoaderData {
  bookings: BookingWithPayments[];
}

export async function clientLoader({}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const result = await AdminService.getBookings("confirmed", new Date());

    return {
      bookings: result.data.bookings,
    };
  } catch (error) {
    handleUnauthorizedError(error);

    return { bookings: [] };
  }
}

export default function PendingBookings({ loaderData }: Route.ComponentProps) {
  const { bookings } = loaderData;

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
          </tr>
        ))}
      </table>
    </>
  );
}
