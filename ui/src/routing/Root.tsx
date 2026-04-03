import { useEffect } from 'react'
import { Outlet } from 'react-router-dom'
import App from '../App'
import { AudioComponents } from '../components/AudioComponents'
import { EpisodeSearchModal } from '../components/EpisodeSearchModal'
import { Header } from '../components/Header'
import { MainContentPanel } from '../components/MainContentPanel'
import { Sidebar } from '../components/Sidebar'
import { $api } from '../utils/http'
import { connectSocket } from '../utils/socketio'


export const Root = () => {
    const user = $api.useQuery('get', '/api/v1/users/{username}', {
        params: { path: { username: 'me' } }
    })

    useEffect(() => {
        if (user.data) {
            connectSocket(user.data.apiKey || '')
        }
    }, [user.data])

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
