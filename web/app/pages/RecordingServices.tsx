import "./RecordingServices.css";
import type { Route } from "./+types/RecordingServices";
import Section from "../components/Section";
import { Link } from "react-router";

export function meta(): Route.MetaDescriptors {
  return [{ title: "녹음 서비스 | 드림하우스 합주실" }];
}

export default function RecordingServices() {
  return (
    <>
      <div className="introduction">
        <div className="spacer" />
        <div className="contents">
          <p>
            드림하우스 합주실에서는 합리적인 비용에 전문적인 퀄리티의 녹음
            서비스를 제공해드리고 있습니다.
          </p>
        </div>
      </div>
      <Section id="equipments" title="장비 소개">
        <small>
          합주실 기본 장비 구성은 <Link to="/#equipments">소개</Link> 페이지를
          참고해 주십시오.
        </small>
        <ul>
          <li>
            레코딩 인터페이스:{" "}
            <Link to="https://rme-audio.de/fireface-ufx-3.html">
              RME UFX III
            </Link>
          </li>
          <li>
            마이크 프리앰프:
            <ul>
              <li>
                <Link to="https://rme-audio.de/12mic.html">RME 12Mic</Link>{" "}
                (12채널)
              </li>
              <li>
                <Link to="https://www.baeaudio.com/products/1073mp">
                  BAE 1073MP
                </Link>{" "}
                (Neve 1073계열, 2채널)
              </li>
              <li>
                <Link to="https://www.cloudmicrophones.com/products/cloudlifter-cl-z">
                  Cloudlifter CL-Z
                </Link>
              </li>
            </ul>
          </li>
          <li>
            컴프레서: Audio-Scape Opto Compressor ×2 (LA-2A 계열)
          </li>
          <li>
            마이크 (기본 세팅):
            <ul>
              <li>
                기타 캐비닛 마이크:{" "}
                <Link to="https://www.shure.com/en-US/products/microphones/sm57">
                  Shure SM57
                </Link>
              </li>
              <li>
                드럼 마이크:{" "}
                <Link to="https://seelectronics.com/products/v-pack-arena/">
                  SE Electronics V Pack Arena
                </Link>
                <ul>
                  <li>스네어 탑: SE V7 X</li>
                  <li>스네어 바텀: Shure SM57</li>
                  <li>탐: SE V BEAT</li>
                  <li>베이스 드럼: SE V KICK</li>
                  <li>
                    하이햇:{" "}
                    <Link to="https://www.shure.com/en-US/products/microphones/sm81?variant=SM81-LC">
                      Shure SM81
                    </Link>
                  </li>
                  <li>오버헤드: SE sE8</li>
                </ul>
              </li>
            </ul>
          </li>
          <li>
            마이크 (녹음용):
            <ul>
              <li>
                <Link to="https://www.lautenaudio.com/eden-lt-386">
                  Lauten Audio LT-386 Eden
                </Link>{" "}
                (진공관 컨덴서)
              </li>
              <li>
                <Link to="https://www.shure.com/en-US/products/microphones/sm7b">
                  Shure SM7B
                </Link>{" "}
                (다이나믹)
              </li>
              <li>
                <Link to="https://products.electrovoice.com/na/en/re20">
                  Electro-Voice RE20
                </Link>{" "}
                (다이나믹)
              </li>
              <li>
                <Link to="https://www.neumann.com/ko-kr/products/microphones/tlm-103">
                  Neumann TLM 103
                </Link>{" "}(컨덴서)
              </li>
              <li>
                <Link to="https://www.telefunken-elektroakustik.com/product/tf51/">Telefunken TF51</Link>{" "}
                (진공관 컨덴서)
              </li>
              <li>
                <Link to="https://soyuzmicrophones.com/013-series">Soyuz Microphones 013 FET</Link>{" "}
                ×2 (컨덴서)
              </li>
              <li>
                <Link to="https://www.lautenaudio.com/ls-208">
                  Lauten Audio LS-208
                </Link>{" "}
                (컨덴서)
              </li>
              <li>
                <Link to="https://www.akg.com/3059X00060.html">
                  AKG C414 XLII
                </Link>{" "}
                ×2 (컨덴서)
              </li>
            </ul>
          </li>
          <li>
            리플렉션 필터:{" "}
            <Link to="https://seelectronics.com/products/rf-pro/">
              SE Electronics RF Pro
            </Link>
          </li>
        </ul>
      </Section>
      <Section id="specs" title="녹음 스펙">
        <ul>
          <li>총 42채널의 멀티트랙 동시 녹음이 가능합니다.</li>
          <li>샘플링 레이트는 96KHz입니다.</li>
          <li>
            DAW는 Cubase 14 Pro를 사용합니다. 프로젝트 파일은 AAF, DAWProject
            또는 CPR (Cubase 14 이상 필요) 형식으로 전달해 드립니다.
          </li>
        </ul>
      </Section>
      <Section id="guide" title="이용 안내">
        <ul>
          <li>
            기본적인 마이킹 셋업과 녹음 환경은 엔지니어가 미리 세팅해 드립니다.
          </li>
          <li>
            이용하시기 전에 어떤 스타일의 음악을 녹음할 예정인지, 무슨 장비를
            사용하고 싶으신지 등을 미리 알려주시면 도움이 됩니다.
          </li>
          <li>
            PC를 직접 가져오셔서 인터페이스를 연결하여 사용하시는 DAW로 직접
            녹음하시는 것도 가능합니다. 이 경우{" "}
            <Link to="https://rme-audio.de/downloads.html">
              RME Audio의 다운로드 페이지
            </Link>
            에서 Fireface UFX III 드라이버를 미리 설치해 오시면 됩니다.
          </li>
          <li>
            녹음시 유선 이어폰 또는 헤드폰을 직접 가져오시는 것을 권장합니다.
          </li>
        </ul>
      </Section>
      <Section id="pricing" title="이용 요금">
        이용요금은 엔지니어 상주시 시간당 ₩50,000이며, 종일 이용시 최대
        ₩250,000입니다.
      </Section>
      <Section id="inquiries" title="예약 문의">
        녹음 서비스를 이용하실 때는 사전에 시간을 조율해야 합니다. 궁금하신 점이
        있거나 이용을 원하실 경우 <Link to="/inquiries">문의</Link> 바랍니다.
      </Section>
    </>
  );
}
