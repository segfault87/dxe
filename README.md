# DXE

[드림하우스 합주실]의 웹사이트, 예약 시스템, 자동화 시스템 등 전체 인프라스트럭쳐의 소스 코드입니다.

## 모듈 설명

* `backend/types`: 기본 타입 정의
* `backend/data`: 영속성 레이어
* `backend/extern`: 외부 서비스 (e.g., 카카오 API)들에 대한 클라이언트 모음
* `backend/server`: 메인 서버
* `backend/space-coordinator`: 공간 관리자
* `backend/s2s-shared`: 서버 - 공간 관리자간 공유 모듈
* `osd`: On-Site Display 안드로이드 앱
* `web`: 웹 프론트엔드

## 라이센스

[MIT License]를 따릅니다.

자유롭게 포크하여 사용하실 수 있으나 실제 서비스에 활용하실 경우 브랜딩과 사업자 정보를 모두 교체한 뒤 사용해 주십시오.

[드림하우스 합주실]: https://dream-house.kr
[MIT License]: https://opensource.org/license/mit
