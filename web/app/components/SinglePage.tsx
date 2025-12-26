import React from "react";
import { Link } from "react-router";

import "./SinglePage.css";
import LogoType from "../assets/logotype.svg";

export default function SinglePage({
  children,
}: {
  children: React.ReactNode[] | React.ReactNode;
}) {
  return (
    <div className="single-page">
      <Link to="/">
        <img className="logo" src={LogoType} alt="드림하우스 합주실" />
      </Link>
      <div className="contents">{children}</div>
    </div>
  );
}
