import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { getSideBarItems, SideBarItem } from "../services/SideBarItemsService";
import { useCookies } from "react-cookie";

// Menu items.
const items: SideBarItem[] = getSideBarItems();

export function AppSidebar() {
  const location = window.location.pathname;
  const [cookies] = useCookies();

  return (
    <Sidebar collapsible="icon" className="w-50">
      <SidebarContent className="overflow-hidden">
        <SidebarGroup>
          <SidebarGroupLabel className="flex justify-between w-full">
            <span>Application</span>
            {cookies.sidebar_state && <SidebarTrigger />}
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton data-active={item.url === location} asChild>
                    <a className="transition-colors hover:transition-none" href={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}