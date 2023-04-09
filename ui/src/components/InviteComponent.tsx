import {useParams} from "react-router-dom";
import {useEffect, useState} from "react";
import axios from "axios";
import {apiURL, formatTime} from "../utils/Utilities";
import {Loading} from "./Loading";
import {SubmitHandler, useForm} from "react-hook-form";
import {LoginData} from "./LoginComponent";
import {useTranslation} from "react-i18next";


export const InviteComponent = ()=>{
        const params = useParams()
        const [invite, setInvite] = useState<Invite>()
        const [errored, setErrored] = useState<boolean>(false)
        const {register, handleSubmit, formState: {}} = useForm<LoginData>();
        const {t} = useTranslation()

        type Invite = {
            id: string,
            role: string,
            createdAt: string,
            acceptedAt: string,
            expiresAt: string
        }

        useEffect(()=>{
            axios.get(apiURL+"/users/invites/"+params.id).then((res)=>{
                setInvite(res.data)
                })
                .catch(()=>{
                    setErrored(true)
                })
        },[])

        if(!invite&& !errored){
            return <Loading/>
        }

        const onSubmit: SubmitHandler<LoginData> = (data)=>{
            axios.post(apiURL+"/users/", {
                username: data.username,
                password: data.password,
                inviteId: params.id
            }).then((res)=>{
                console.log(res)
            }).catch((e)=>{
                console.log(e)
            })
        }

        if (!invite) {
            return <Loading/>
        }

        return <section className="bg-gray-50 dark:bg-gray-900 h-full">
            <div className="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0">
                <a href="#" className="flex items-center mb-6 text-2xl font-semibold text-gray-900 dark:text-white">
                    <i className="fa-solid fa-music mr-5"></i>
                    PodFetch
                </a>
                <div
                    className="w-full bg-white rounded-lg shadow dark:border md:mt-0 sm:max-w-md xl:p-0 dark:bg-gray-800 dark:border-gray-700">
                    <div className="p-6 space-y-4 md:space-y-6 sm:p-8">
                        <h1 className="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl dark:text-white">
                            {t('create-account-podfetch')}
                        </h1>
                        <div className="grid place-items-center">
                <form onSubmit={handleSubmit(onSubmit)}>
                <div className="grid grid-cols-2 gap-5 text-white">
                    <div>
                        Rolle
                    </div>
                    <div>
                        {invite.role}
                    </div>
                    <div>
                        Erstellt
                    </div>
                    <div>
                        {formatTime(invite.createdAt)}
                    </div>
                    <div>
                        Läuft ab
                    </div>
                    <div>
                        {formatTime(invite.expiresAt)}
                    </div>
                    <label htmlFor="username"
                           className="block pt-2.5 pb-2.5 text-white">{t('username')!}</label>
                    <input type="username" {...register('username', {required: true})} id="username"
                           autoComplete="username"
                           className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                           placeholder={t('your-username')!}/>

                    <label htmlFor="password"
                           className="block pt-2.5 pb-2.5 text-white">{t('password')}</label>
                    <input type="password" id="password" autoComplete="current-password"
                           placeholder="••••••••" {...register('password', {required: true})}
                           className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"/>
                    </div>
                    <button type="submit" className="text-center bg-blue-700 w-full mt-5 pt-1 pb-1 text-white">Absenden</button>
                </form>
    </div>
                    </div>
                </div>
            </div>
        </section>
}
