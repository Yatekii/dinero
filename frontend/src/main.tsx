import React from "react";
import ReactDOM from "react-dom/client";
import "./index.css";
import { Layout } from "./Layout.tsx";
import {
  Route,
  createBrowserRouter,
  createRoutesFromElements,
  RouterProvider,
} from "react-router-dom";
import Dashboard, { dataLoader } from "./dashboard/Dashboard.tsx";
import { Ledger, ledgerLoader } from "./ledger/LedgerView.tsx";
import { LedgerFileView, ledgerFileLoader } from "./ledger/LedgerFileView.tsx";
import { LedgerLayout, ledgersLoader } from "./ledger/LedgerLayout.tsx";
import { LedgerCreate } from "./ledger/LedgerCreate.tsx";
import {
  LedgerOverview,
  ledgerOverviewLoader,
} from "./ledger/LedgerOverview.tsx";
import { ledgerMetaLoader, LedgerMetaView } from "./ledger/LedgerMeta.tsx";

export const API_URL = import.meta.env.PROD
  ? `/api`
  : `http://127.0.0.1:3000/api`;

const router = createBrowserRouter(
  createRoutesFromElements(
    <Route path={"/"} element={<Layout />}>
      <Route path="/" element={<Dashboard />} loader={dataLoader} />
      <Route path={"ledger"} element={<LedgerLayout />} loader={ledgersLoader}>
        <Route
          path={"/ledger"}
          element={<LedgerOverview />}
          loader={ledgerOverviewLoader}
        />
        <Route path={"/ledger/add"} element={<LedgerCreate />} />
        <Route path={"/ledger/:ledgerId"}>
          <Route
            path={"/ledger/:ledgerId/"}
            loader={ledgerLoader}
            element={<Ledger />}
          />
          <Route
            path={"/ledger/:ledgerId/files"}
            loader={ledgerFileLoader}
            element={<LedgerFileView />}
          />
          <Route
            path={"/ledger/:ledgerId/meta"}
            loader={ledgerMetaLoader}
            element={<LedgerMetaView />}
          />
        </Route>
      </Route>
    </Route>
  )
);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
