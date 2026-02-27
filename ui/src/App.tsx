import {FC, PropsWithChildren, Suspense} from 'react'
import {createBrowserRouter, createRoutesFromElements, Navigate, Route} from 'react-router-dom'
import {
    EpisodeSearchViewLazyLoad,
    HomepageViewLazyLoad,
    PlaylistViewLazyLoad,
    PodcastDetailViewLazyLoad,
    PodcastInfoViewLazyLoad,
    PodcastViewLazyLoad,
    SettingsViewLazyLoad,
    StatisticsViewLazyLoad,
    TimeLineViewLazyLoad,
    UserAdminViewLazyLoad
} from "./utils/LazyLoading"
import {Root} from "./routing/Root"
import {AcceptInvite} from "./pages/AcceptInvite"
import {Login} from "./pages/Login"
import "./App.css"
import './App.css'
import {HomePageSelector} from "./pages/HomePageSelector";
import {PlaylistPage} from "./pages/PlaylistPage";
import {Settings} from "./components/SettingsData";
import {SettingsOPMLExport} from "./components/SettingsOPMLExport";
import {SettingsNaming} from "./components/SettingsNaming";
import {SettingsPodcastDelete} from "./components/SettingsPodcastDelete";
import {UserAdminUsers} from "./components/UserAdminUsers";
import {UserAdminInvites} from "./components/UserAdminInvites";
import {UserManagementPage} from "./pages/UserManagement";
import {GPodderIntegration} from "./pages/GPodderIntegration";
import {TagsPage} from "./pages/TagsPage";

export const router = createBrowserRouter(createRoutesFromElements(
    <>
        <Route path="/" element={<Root/>}>
            <Route index element={<Navigate to="home"/>}/>
            <Route path="home" element={<HomePageSelector/>}>
                <Route index element={<Navigate to="view"/>}/>
                <Route path="view" element={<Suspense><HomepageViewLazyLoad /></Suspense>}/>
                <Route path={"playlist"}>
                    <Route index element={<Suspense><PlaylistPage/></Suspense>}></Route>
                    <Route path={":id"} element={<Suspense><PlaylistViewLazyLoad/></Suspense>}/>
                </Route>
            </Route>
            <Route path={"podcasts"}>
                <Route index element={<Suspense><PodcastViewLazyLoad/></Suspense>}/>
                <Route path={":id/episodes"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
                <Route path={":id/episodes/:podcastid"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
                <Route path={"search"} element={<Suspense><EpisodeSearchViewLazyLoad/></Suspense>}/>
            </Route>

            <Route path="timeline" element={<Suspense><TimeLineViewLazyLoad /></Suspense>} />
            <Route path="stats" element={<Suspense><StatisticsViewLazyLoad /></Suspense>} />
            <Route path={"favorites"}>
                <Route element={<PodcastViewLazyLoad onlyFavorites={true} />} index />
                <Route path={":id/episodes"} element={<Suspense><PodcastDetailViewLazyLoad /></Suspense>} />
                <Route path={":id/episodes/:podcastid"} element={<Suspense><PodcastDetailViewLazyLoad /></Suspense>} />
            </Route>
            <Route path={"info"} element={<Suspense><PodcastInfoViewLazyLoad /></Suspense>} />
            <Route path={"settings"} element={<Suspense><SettingsViewLazyLoad /></Suspense>}>
                <Route index element={<Navigate  to="retention"/>}/>
                <Route path="retention" element={<Settings/>}/>
                <Route path="opml" element={<SettingsOPMLExport/>}/>
                <Route path="naming" element={<SettingsNaming/>}/>
                <Route path="podcasts" element={<SettingsPodcastDelete/>}/>
                <Route path="gpodder" element={<GPodderIntegration/>}/>
            </Route>
            <Route path={"administration"} element={<Suspense><UserAdminViewLazyLoad /></Suspense>}>
                <Route index element={<Navigate to="users"/>}/>
                <Route path="users" element={<UserAdminUsers/>}/>
                <Route path="invites" element={<UserAdminInvites/>}/>
            </Route>
            <Route path={"profile"}>
                <Route index element={<UserManagementPage/>}/>
            </Route>
            <Route path="tags">
                <Route index element={<TagsPage/>}/>
            </Route>
        </Route>
        <Route path="/login" element={<Login />} />
        <Route path="/invite/:id" element={<AcceptInvite />}></Route>
    </>
), {
    basename: import.meta.env.BASE_URL
})

const App: FC<PropsWithChildren> = ({ children }) => {

    return (
        <Suspense>
            {children}
        </Suspense>
    )
}

export default App
