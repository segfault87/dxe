import { useState } from "react";
import { useNavigate } from "react-router";

import type { Route } from "./+types/AdhocParkings";
import AdminService from "../../api/admin";
import { DEFAULT_SPACE_ID } from "../../constants";
import { toUtcIso8601 } from "../../lib/datetime";
import checkPlateNumber from "../../lib/PlateNumber";
import { defaultErrorHandler, loaderErrorHandler } from "../../lib/error";
import type { AdhocParkingId } from "../../types/models/base";
import type { AdhocParking } from "../../types/models/booking";

interface LoaderData {
  parkings: AdhocParking[];
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const result = await AdminService.getAdhocParkings(DEFAULT_SPACE_ID);

    return {
      parkings: result.data.parkings,
    };
  } catch (error) {
    throw loaderErrorHandler(error, request.url);
  }
}

export default function AdhocParkings({ loaderData }: Route.ComponentProps) {
  const { parkings } = loaderData;
  const navigate = useNavigate();

  const deleteParking = async (parkingId: AdhocParkingId) => {
    if (!confirm("정말로 삭제하시겠습니까?")) {
      return;
    }

    try {
      await AdminService.deleteAdhocParking(parkingId);
      navigate(0);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  const [startTime, setStartTime] = useState(toUtcIso8601(new Date()));
  const [desiredHours, setDesiredHours] = useState("2");
  const [licensePlateNumber, setLicensePlateNumber] = useState("");

  const createParking = async () => {
    try {
      await AdminService.createAdhocParking({
        spaceId: DEFAULT_SPACE_ID,
        timeFrom: startTime,
        desiredHours: parseInt(desiredHours),
        licensePlateNumber: licensePlateNumber,
      });

      navigate(0);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  return (
    <>
      <h2>임의 주차 목록</h2>
      <table>
        <tr>
          <th>시작시간</th>
          <th>종료시간</th>
          <th>차량번호</th>
          <th>동작</th>
        </tr>
        {parkings.map((v) => (
          <tr key={v.id}>
            <td>{new Date(v.timeFrom).toLocaleString()}</td>
            <td>{new Date(v.timeTo).toLocaleString()}</td>
            <td>{v.licensePlateNumber}</td>
            <td>
              <button className="primary" onClick={() => deleteParking(v.id)}>
                삭제
              </button>
            </td>
          </tr>
        ))}
      </table>
      <h2>새 주차 생성</h2>
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
      차량번호:{" "}
      <input
        type="text"
        value={licensePlateNumber}
        onChange={(e) => {
          setLicensePlateNumber(e.target.value);
        }}
      />
      <br />
      <button
        onClick={createParking}
        disabled={checkPlateNumber(licensePlateNumber) === false}
      >
        생성
      </button>
    </>
  );
}
