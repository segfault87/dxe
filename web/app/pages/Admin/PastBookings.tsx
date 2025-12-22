import { Link } from "react-router";

import type { Route } from "./+types/PastBookings";
import AdminService from "../../api/admin";
import { handleUnauthorizedError } from "../../lib/error";
import type { BookingWithPayments } from "../../types/models/booking";

const ITEMS_PER_PAGES = 30;

interface LoaderData {
  bookings: BookingWithPayments[];
  page: number;
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const url = new URL(request.url);
    const searchParams = url.searchParams;

    const page = parseInt(searchParams.get("page") ?? "0");
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
      page: page,
    };
  } catch (error) {
    handleUnauthorizedError(error);

    return { bookings: [], page: 0 };
  }
}

export default function PastBookings({ loaderData }: Route.ComponentProps) {
  const { bookings, page } = loaderData;

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
      {page > 0 ? (
        <>
          <Link to={`?page=${page - 1}`}>이전</Link>{" "}
        </>
      ) : null}
      <Link to={`?page=${page + 1}`}>다음</Link>
    </>
  );
}
