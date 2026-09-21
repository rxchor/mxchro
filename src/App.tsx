import { BrowserRouter, Route, Routes } from "react-router-dom";
import "./App.css";
import { SideBarItem, getSideBarItems } from "./services/SideBarItemsService";
import { AppHeader } from "./components/AppHeader";

export default function App() {
  const navBarItems: SideBarItem[] = getSideBarItems();

  return (
    <BrowserRouter>
      <div className="app-content">
        <AppHeader className="app-content-header" />
        <main>
          <Routes>
            {navBarItems.map((item) => (
              <Route
                key={item.title}
                path={item.url}
                element={<item.component />}
              />
            ))}
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}
