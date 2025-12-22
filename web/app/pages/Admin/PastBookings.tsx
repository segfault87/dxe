import { Link } from "react-router";

import type { Route } from "./+types/PendingBookings";
import AdminService from "../../api/admin";
import type { BookingWithPayments } from "../../types/models/booking";
import { handleUnauthorizedError } from "../../lib/error";

const ITEMS_PER_PAGES = 30;

interface LoaderData {
  bookings: BookingWithPayments[];
  page?: number;
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const page = parseInt(params.page ?? "0");
    const offset = page * ITEMS_PER_PAGES;
    const limit = ITEMS_PER_PAGES;
    const result = await AdminService.getBookings(
      "complete",
      undefined,
      undefined,
      limit,
      offset,
    );

    return {
      bookings: result.data.bookings,
    };
  } catch (error) {
    handleUnauthorizedError(error);

    return { bookings: [] };
  }
}

export default function PastBookings({ loaderData }: Route.ComponentProps) {
  const { bookings } = loaderData;

  return (
    <>
      <h2>이용 내역</h2>
      <table>
        <tr>
          <th>ID</th>
          <th>고객명</th>
          <th>시작시간</th>
          <th>종료시간</th>
        </tr>
        {bookings.map((e) => (
          <tr key={e.booking.id}>
            <td>
              <Link to={`/admin/booking/${e.booking.id}`}>{e.booking.id}</Link>
            </td>
            <td>{e.booking.customer.name}</td>
            <td>{new Date(e.booking.bookingStart).toLocaleString()}</td>
            <td>{new Date(e.booking.bookingEnd).toLocaleString()}</td>
          </tr>
        ))}
      </table>
    </>
  );
}
