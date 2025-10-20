import "./Reservation.css";
import type { Route } from "./+types/Reservation";
import Section from "../components/Section";

export function meta({}: Route.MetaArgs) {
  return [{ title: "예약 | 드림하우스 합주실" }];
}

export default function Reservation() {
  return (
    <>
      <div className="reservation">
        <p>현재 예약 시스템은 준비중입니다.</p>
        <p>
          예약 시스템이 만들어지는 동안 아래 Google Forms 링크를 통해
          예약해주시면 감사하겠습니다.
          <br />
          예약이 확정되면 예약자분의 휴대전화로 안내 문자가 발송됩니다.
        </p>
        <p>입금 계좌: 신한은행 110-609-686081 (박준규)</p>
        <a
          className="cta"
          href="https://docs.google.com/forms/d/e/1FAIpQLSfopYWJbchBLD3BTa7dRmFiIeyBR6WQ_-KLUwEks_DS5uL0mQ/viewform?usp=header"
        >
          Google Forms 예약
        </a>
      </div>
      <Section id="calendar" title="예약 현황">
        <iframe src="https://calendar.google.com/calendar/u/0/embed?src=ef224add7910191c13cfee340e4122d7baebf2d62633cf763783473d569cf71f@group.calendar.google.com&ctz=Asia/Seoul" />
      </Section>
    </>
  );
}
