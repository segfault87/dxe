import { Link } from "react-router";
import Slider, { type Settings as SliderSettings } from "react-slick";
import "slick-carousel/slick/slick.css";
import "slick-carousel/slick/slick-theme.css";

import "./Index.css";
import type { Route } from "./+types/Index";
import Image1 from "../assets/interior1.jpg";
import Image2 from "../assets/interior2.jpg";
import Image3 from "../assets/interior3.jpg";
import Image4 from "../assets/interior4.jpg";
import Image5 from "../assets/interior5.jpg";
import Section from "../components/Section";
import { useAuth } from "../context/AuthContext";
import LocationInformation from "../components/LocationInformation";

// @ts-expect-error Workaround for SSR
const SliderComponent = typeof window === "undefined" ? Slider.default : Slider;

export function meta({}: Route.MetaArgs) {
  return [
    { title: "드림하우스 합주실" },
    {
      property: "og:title",
      content: "드림하우스 합주실",
    },
    { property: "og:type", content: "website" },
    { property: "og:image", content: "/og.png" },
    { property: "og:url", content: "https://dream-house.kr" },
    { property: "og:description", content: "직장인 음악인들을 위한 합주 공간" },
    { property: "og:locale", content: "ko_KR" },
  ];
}

function Carousel() {
  const settings: SliderSettings = {
    dots: true,
    fade: true,
    infinite: true,
    speed: 500,
    slidesToShow: 1,
    slidesToScroll: 1,
    waitForAnimate: false,
    autoplay: true,
    autoplaySpeed: 3000,
  };

  return (
    <SliderComponent {...settings}>
      <div className="container">
        <img src={Image1} alt="합주실 내부" />
      </div>
      <div className="container">
        <img src={Image2} alt="대기실" />
      </div>
      <div className="container">
        <img src={Image3} alt="합주실 측면" />
      </div>
      <div className="container">
        <img src={Image4} alt="합주실 측면" />
      </div>
      <div className="container">
        <img src={Image5} alt="대기실" />
      </div>
    </SliderComponent>
  );
}

function Introduction() {
  return (
    <>
      <div className="spacer" />
      <div className="contents">
        <p>
          드림하우스 합주실은 직장인 음악인 중심으로 운영되는 합주실 및 레코딩
          스튜디오입니다. 서울시 강남구 테헤란로에 위치하여 뛰어난 접근성을
          가지고 있습니다.
        </p>
        <p>
          우수한 음향 시설과 철저한 관리를 통해 고객 여러분들의 만족스러운 이용
          경험을 약속합니다.
        </p>
      </div>
    </>
  );
}

export function Actions() {
  const auth = useAuth();

  let action = (
    <Link to="/reservation/" className="cta">
      예약하기
    </Link>
  );

  if (auth) {
    if (auth.activeBookings.default) {
      const activeBooking = auth.activeBookings.default;
      action = (
        <Link to={`/reservation/${activeBooking.id}`} className="cta">
          현재 이용 중 예약 보기 ({activeBooking.customer.name})
        </Link>
      );
    } else if (auth.pendingBookings.default) {
      const today = new Date().toDateString();

      for (const booking of auth.pendingBookings.default) {
        if (new Date(booking.bookingStart).toDateString() === today) {
          action = (
            <Link to={`/reservation/${booking.id}`} className="cta">
              오늘의 예약 ({booking.customer.name})
            </Link>
          );
          break;
        }
      }
    }
  }

  return <div className="actions">{action}</div>;
}

export default function Index() {
  return (
    <>
      <div className="top">
        <Actions />
        <div className="cell availability-info">
          서울특별시 강남구 삼성로 517 채널리저브 B1층 101-2호
          <br />
          연중무휴 24시간 운영
        </div>
      </div>
      <div className="carousel">
        <Carousel />
      </div>
      <div className="introduction">
        <Introduction />
      </div>
      <Section id="equipments" title="시설 소개">
        <ul>
          <li>
            기타 앰프:
            <ul>
              <li>Blackstar Series One 200W Head × Blackstar 4x12 Cabinet</li>
              <li>Fender '65 Twin Reverb</li>
              <li>Line 6 Powercab 112</li>
            </ul>
          </li>
          <li>
            베이스 앰프: <s>Ampeg SVT-2PRO Head</s> × Ampeg SVT-810E Cabinet
            <br />
            (현재 헤드는 수리 중으로 임시로 Aguilar Tone Hammer 350으로
            대체중입니다.)
          </li>
          <li>
            드럼세트:
            <ul>
              <li>Pearl PMX Professional Series</li>
              <li>
                Zildjian K Custom Dark Series Cymbals (14" Hi Hat, 16"/18"
                Crashes, 20" Ride)
              </li>
              <li>Tama Iron Cobra Twin Pedal</li>
            </ul>
          </li>
          <li>키보드: Yamaha MX88</li>
          <li>
            마이크:
            <ul>
              <li>유선: Shure SM58 ×3</li>
              <li>무선: Kanals KB-9700 ×2</li>
            </ul>
          </li>
          <li>PA 스피커: JBL EON 715 ×2</li>
          <li>모니터링 스피커: Behringer Eurolive F1220D</li>
          <li>믹서: Roland VM-3100Pro</li>
        </ul>
        <small>
          적정 정원은 7명이며, 그 이상 입장이 가능하지만 다소 좁을 수 있습니다.
        </small>
      </Section>
      <Section id="pricing" title="이용 요금">
        <ul>
          <li>이용요금은 시간당 ₩20,000입니다.</li>
          <li>
            악기 대여시 별도의 비용이 발생할 수 있습니다. (대여가 필요하실 경우{" "}
            <Link to="/inquiries/">문의</Link> 바랍니다)
          </li>
        </ul>
      </Section>
      <Section id="location" title="오시는 길">
        <LocationInformation buttonClassName="cta" />
      </Section>
    </>
  );
}
