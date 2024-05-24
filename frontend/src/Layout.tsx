import { Card, Tab, TabGroup, TabList } from "@tremor/react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";

const items = () => [
  { name: "Overview", url: "/" },
  { name: "Ledgers", url: "/ledger" },
];

const reverseItems = () => items().reverse();

export function Layout() {
  return (
    <Card className="mx-full h-full">
      <MainMenu />
      <Outlet />
    </Card>
  );
}

function MainMenu() {
  const navigate = useNavigate();
  const location = useLocation();

  const reverse = reverseItems();
  const index =
    reverse.length -
    1 -
    reverse.findIndex((item) => location.pathname.startsWith(item.url));
  const menuItems = items();
  return (
    <>
      <TabGroup
        index={index}
        onIndexChange={(index) => navigate(menuItems[index].url)}
        className="mb-3"
      >
        <TabList className="mt-4">
          {menuItems.map((item, index) => (
            <Tab tabIndex={index} key={index}>
              {item.name}
            </Tab>
          ))}
        </TabList>
      </TabGroup>
    </>
  );
}
