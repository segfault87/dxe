export default function LocationInformation({
  buttonClassName,
}: {
  buttonClassName: string;
}) {
  return (
    <>
      <div>
        <a className={buttonClassName} href="https://naver.me/5u9yYPi2">
          네이버 지도
        </a>{" "}
        <a className={buttonClassName} href="https://kko.kakao.com/g9cAKvMpgq">
          카카오맵
        </a>{" "}
        <a className={buttonClassName} href="https://tmap.life/0fbed987">
          티맵
        </a>{" "}
        <a
          className={buttonClassName}
          href="https://maps.app.goo.gl/MRiauyo4mNgGQfK47"
        >
          Google Maps
        </a>
      </div>
      <p>
        <ul>
          <li>
            차량 이동 시: 채널리저브 지하주차장에 정차하신 후 지하 3층에 있는
            상가 엘리베이터를 타고 지하 1층으로 올라와 주세요.
          </li>
          <li>
            도보 이동 시: 대로변에서 오실 경우 씨유 편의점 오른편에 있는
            계단으로 채널리저브 상가 지하 1층으로 들어오세요. 상가 1층 내부에서
            엘리베이터를 타고 지하 1층으로 내려오실 수도 있습니다.
          </li>
          <li>
            지하 1층 상가에서 세탁소를 지나서 솔섬식품이 보이면 왼편 안쪽으로
            들어오세요. 가장 끝에 있는 비상계단 오른쪽이 출입구입니다.
          </li>
        </ul>
      </p>
    </>
  );
}
