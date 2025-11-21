import type { Route } from "./+types/Users";
import AdminService from "../../api/admin";
import type { SelfUser } from "../../types/models/user";

interface LoaderData {
  users: SelfUser[];
}

export async function clientLoader({}: Route.ClientLoaderArgs): Promise<LoaderData> {
  const result = await AdminService.getUsers();

  return {
    users: result.data.users,
  };
}

export default function PendingBookings({ loaderData }: Route.ComponentProps) {
  const { users } = loaderData;

  return (
    <>
      <h2>전체 이용자 목록</h2>
      <table>
        <tr>
          <th>아이디</th>
          <th>이름</th>
          <th>가입일시</th>
          <th>차량번호</th>
        </tr>
        {users.map((e) => (
          <tr key={e.id}>
            <td>{e.id}</td>
            <td>{e.name}</td>
            <td>{new Date(e.createdAt).toLocaleString()}</td>
            <td>{e.licensePlateNumber ?? ""}</td>
          </tr>
        ))}
      </table>
    </>
  );
}
