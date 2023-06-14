import {Dropdown} from "./I18nDropdown"
import {Notifications} from "./Notifications"
import {UserMenu} from "./UserMenu"

export const Header = ()=>{
    

    return (
        <div className="flex items-center justify-end gap-8 border-gray-100 mb-8 py-6">
            <Dropdown/>
            <Notifications/>
            <div className="hidden xs:block border-r border-r-stone-200 h-full w-1"></div>
            <UserMenu/>
        </div>
    )
}
