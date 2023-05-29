import {FC} from "react";

type PodFetchLogoProps = {
    className?: string
}

export const PodFetchLogo:FC<PodFetchLogoProps> = ({className}) => {
    return <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960" className={"h-9 fill-amber-600 "+className}>
        <path d="m200-584-30-66-66-30 66-30 30-66 30 66 66 30-66 30-30 66Zm520-120-17-39-39-17 39-17 17-39 17 39 39 17-39 17-17 39Zm80 160-17-39-39-17 39-17 17-39 17 39 39 17-39 17-17 39ZM480-383q-43 0-72-31t-29-75v-251q0-42
         29.5-71t71.5-29q42 0 71.5 29t29.5 71v251q0 44-29 75t-72 31Zm0-228ZM450-80v-136q-106-11-178-89t-72-184h60q0 91 64.5
          153T480-274q91 0 155.5-62T700-489h60q0 106-72 184t-178 89v136h-60Zm30-363q18 0 29.5-13.5T521-489v-251q0-17-12-28.5T480-780q-17
           0-29 11.5T439-740v251q0 19 11.5 32.5T480-443Z"/>
    </svg>
}
