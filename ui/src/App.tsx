import './App.css'
import {BrowserRouter, Route, Routes} from "react-router-dom";
import {SideBar} from "./components/SideBar";
import {Header} from "./components/Header";
import {useAppSelector} from "./store/hooks";
import {Podcasts} from "./pages/Podcasts";
import {PodcastDetailPage} from "./pages/PodcastDetailPage";
import {Modal} from "./components/Modal";

const App = ()=> {
    const sideBarCollapsed = useAppSelector(state=>state.common.sideBarCollapsed)

    return (
      <BrowserRouter basename="/ui">
          <div className="grid  grid-rows-[auto_1fr] h-full md:grid-cols-[300px_1fr]">
              <Header/>
              <SideBar/>
              <div className={`col-span-6 md:col-span-5 ${sideBarCollapsed?'xs:col-span-5':'hidden'} md:block w-full overflow-x-auto`}>
                  <Routes>
                      <Route path={"/home"} element={<div>test</div>}/>
                      <Route path={"/podcasts"} element={<Podcasts/>}/>
                      <Route path={"/podcasts/:id"} element={<PodcastDetailPage/>}/>
                  </Routes>
              </div>
          </div>
      </BrowserRouter>
  )
}

export default App
