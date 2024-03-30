import React from "react";
import ReactDOM from "react-dom/client";
import Dashboard from "./Dashboard.tsx";
import { Ledger, ledgerLoader } from "./Ledger.tsx";
import "./index.css";
import { createBrowserRouter, RouterProvider } from "react-router-dom";

const router = createBrowserRouter([
  {
    path: "/",
    element: <Dashboard />,
  },
  {
    path: "/ledger/:ledgerId",
    element: <Ledger />,
    loader: ledgerLoader,
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
