import {Dropdown} from "./I18nDropdown"
import {Notifications} from "./Notifications"
import {UserMenu} from "./UserMenu"

export const Header = ()=>{
    

    return (
        <div className="flex items-center justify-between gap-8 border-gray-100 mb-8 px-8 py-6">
            

            <div className="flex-1 flex items-center justify-end gap-8 border-r border-r-stone-200 h-full pr-8">
                <Dropdown/>
                <Notifications/>
            </div>

            <UserMenu/>
        </div>
    )
}
