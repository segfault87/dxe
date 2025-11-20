import axios from "axios";

export default axios.create({
  baseURL: "/api",
  headers: {
    "Content-Type": "application/json; charset=utf-8",
  },
});
