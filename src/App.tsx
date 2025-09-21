import { BrowserRouter, Route, Routes } from 'react-router-dom';
import './App.css';
import { SideBarItem, getSideBarItems } from './services/SideBarItemsService';
import { SidebarProvider, SidebarTrigger } from './components/ui/sidebar';
import { AppSidebar } from './components/side_bar/AppSideBar';
import { useCookies } from 'react-cookie'

export default function App() {
    const navBarItems: SideBarItem[] = getSideBarItems();
    const [cookies] = useCookies(["sidebar_state"]);

    return(
      <SidebarProvider defaultOpen={cookies.sidebar_state === true}>
        <AppSidebar />
        <SidebarTrigger />
        <main>
          <BrowserRouter>
            <Routes>
                  {navBarItems.map((item) => (
                    <Route key={item.title} path={item.url} element={<item.component />} />
                  ))}
            </Routes>
          </BrowserRouter>
        </main>
      </SidebarProvider>
    );
}