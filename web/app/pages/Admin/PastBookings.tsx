import { Link, useNavigate } from "react-router";
import type { NavigateFunction } from "react-router";

import type { Route } from "./+types/PastBookings";
import AdminService from "../../api/admin";
import { handleUnauthorizedError } from "../../lib/error";
import type { BookingWithPayments } from "../../types/models/booking";
import type { BookingId } from "../../types/models/base";

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

async function refundBooking(id: BookingId, navigate: NavigateFunction) {
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

export default function PastBookings({ loaderData }: Route.ComponentProps) {
  const { bookings, page } = loaderData;

  const navigate = useNavigate();

  return (
    <>
      <h2>이용 내역</h2>
      <table>
        <tr>
          <th>ID</th>
          <th>고객명</th>
          <th>시작시간</th>
          <th>종료시간</th>
          <th>동작</th>
        </tr>
        {bookings.map((e) => (
          <tr key={e.booking.id}>
            <td>
              <Link to={`/admin/booking/${e.booking.id}`}>{e.booking.id}</Link>
            </td>
            <td>{e.booking.customer.name}</td>
            <td>{new Date(e.booking.bookingStart).toLocaleString()}</td>
            <td>{new Date(e.booking.bookingEnd).toLocaleString()}</td>
            <td><button onClick={() => refundBooking(e.booking.id, navigate)}>환불</button></td>
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
