import { useEffect } from 'react'
import { Outlet, useNavigate } from 'react-router-dom'
import useCommon from '../store/CommonSlice'
import App from '../App'
import { AudioComponents } from '../components/AudioComponents'
import { EpisodeSearchModal } from '../components/EpisodeSearchModal'
import { Header } from '../components/Header'
import { MainContentPanel } from '../components/MainContentPanel'
import { Sidebar } from '../components/Sidebar'


export const Root = () => {
    return (
        <App>
            <div className="grid grid-cols-[1fr] md:grid-cols-[18rem_1fr] grid-rows-[1fr_auto]">
                <Sidebar />
                <MainContentPanel>
                    <Header />
                    <div className="grid grid-rows-[1fr_auto] pb-8">
                        <Outlet />
                    </div>
                </MainContentPanel>
                <AudioComponents />
                <EpisodeSearchModal />
            </div>
        </App>
    )
}
