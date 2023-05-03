import {lazy} from "react";

export const PodcastViewLazyLoad = lazy(()=>import('../pages/Podcasts').then(module=> {
    return{default:module["Podcasts"]}
}))

export const PodcastDetailViewLazyLoad = lazy(()=>import('../pages/PodcastDetailPage').then(module=> {
    return{default:module["PodcastDetailPage"]}
}))

export const PodcastInfoViewLazyLoad = lazy(()=>import('../pages/PodcastInfoPage').then(module=> {
    return{default:module["PodcastInfoPage"]}
}))

export const SettingsViewLazyLoad = lazy(()=>import('../pages/SettingsPage').then(module=> {
    return{default:module["SettingsPage"]}
}))

export const AdministrationViewLazyLoad = lazy(()=>import('../pages/AdministrationPage').then(module=> {
    return{default:module["AdministrationPage"]}
}))

export const AdministrationUserViewLazyLoad = lazy(()=>import('../pages/AdministrationUserPage').then(module=> {
    return{default:module["AdministrationUserPage"]}
}))

export const InviteAdministrationUserViewLazyLoad = lazy(()=>import('../pages/InviteAdministrationUserPage').then(module=> {
    return{default:module["InviteAdministrationUserPage"]}
}))

export const TimeLineViewLazyLoad = lazy(()=>import('../pages/Timeline').then(module=> {
    return{default:module["Timeline"]}
}))
export const MobileSearchViewLazyLoad = lazy(()=>import('../pages/MobileSearchPage').then(module=> {
    return{default:module["MobileSearchPage"]}
}))
