import "./Inquiries.css";
import type { Route } from "./+types/Inquiries";
import Section from "../components/Section";

export function meta({}: Route.MetaArgs) {
  return [{ title: "문의 | 드림하우스 합주실" }];
}

export default function Inquiries() {
  return (
    <>
      <div className="inquiries">
        <p>
          시설 이용과 관련하여 문의사항이 있거나, 요청이 필요한 경우 아래
          연락처를 통해 문의해주시면 감사하겠습니다.
        </p>
        <a className="cta" href="mailto:segfault87+dxe@gmail.com">
          이메일
        </a>
        <a className="cta" href="http://pf.kakao.com/_xfUQLn/chat">
          카카오톡
        </a>
        <a className="cta" href="tel:+8250219445150">
          전화
        </a>
      </div>
      <Section id="biz-info" title="사업자 정보">
        <ul>
          <li>사업자명: 디엑스이 스튜디오</li>
          <li>대표자: 박준규</li>
          <li>사업자번호: 701-07-03619</li>
        </ul>
      </Section>
    </>
  );
}
