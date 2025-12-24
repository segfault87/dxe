import type { Route } from "./+types/Groups";
import AdminService from "../../api/admin";
import { loaderErrorHandler } from "../../lib/error";
import type { GroupWithUsers } from "../../types/models/group";

interface LoaderData {
  groups: GroupWithUsers[];
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const result = await AdminService.getGroups();

    return {
      groups: result.data.groups,
    };
  } catch (error) {
    throw loaderErrorHandler(error, request.url);
  }
}

export default function PendingBookings({ loaderData }: Route.ComponentProps) {
  const { groups } = loaderData;

  return (
    <>
      <h2>전체 그룹 목록</h2>
      <table>
        <tr>
          <th>아이디</th>
          <th>이름</th>
          <th>생성 일시</th>
          <th>오픈 여부</th>
          <th>구성원</th>
        </tr>
        {groups.map((e) => (
          <tr key={e.id}>
            <td>{e.id}</td>
            <td>{e.name}</td>
            <td>{new Date(e.createdAt).toLocaleString()}</td>
            <td>{e.isOpen ? "Y" : "N"}</td>
            <td>
              <ul>
                {e.users.map((e) => (
                  <li key={e.id}>{e.name}</li>
                ))}
              </ul>
            </td>
          </tr>
        ))}
      </table>
    </>
  );
}
