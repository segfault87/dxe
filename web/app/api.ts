import axios from "axios";

export default axios.create({
  baseURL: "/api",
  maxRedirects: 0,
  headers: {
    "Content-Type": "application/json; charset=utf-8",
  },
});
