import React from 'react'
import ReactDOM from 'react-dom/client'
import {router} from './App'
import './index.css'
import {Provider} from "react-redux";
import {store} from "./store/store";
import {I18nextProvider} from "react-i18next";
import i18n from "./language/i18n";
import "@fortawesome/fontawesome-free/css/all.min.css";
import "@fontsource/roboto"
import "@fontsource/anton"
import {RouterProvider} from "react-router-dom";

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
      <I18nextProvider i18n={i18n}>
          <Provider store={store}>
              <RouterProvider router={router}/>
          </Provider>
      </I18nextProvider>
  </React.StrictMode>,
)
