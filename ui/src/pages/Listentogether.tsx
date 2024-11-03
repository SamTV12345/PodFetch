import {CustomInput} from "../components/CustomInput";
import {useForm} from "react-hook-form";
import { SlArrowRight } from "react-icons/sl";
import axios from "axios";


type FormListenProps = {
    roomName: string
}

export const ListenTogether = ()=>{
    const {handleSubmit} = useForm<FormListenProps>()


    const onSubmit = (data: FormListenProps) => {
        axios.get("/listen-together/")
    }

    return <form className="grid place-items-center bg-radial-mustard h-full" onSubmit={handleSubmit(onSubmit)}>
        <div className="">
            <h1 className="text-xl">Listen Together with your friends</h1>
            <span className="relative">
            <CustomInput className="w-full" placeholder={"Enter a room name"} />
                <button type="submit"><SlArrowRight className="right-5 absolute top-1/4" /></button>
            </span>
        </div>
    </form>
}
