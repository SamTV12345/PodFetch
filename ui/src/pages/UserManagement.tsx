import {FC, useEffect} from "react";
import {useTranslation} from "react-i18next";
import {Heading1} from "../components/Heading1";
import {CustomInput} from "../components/CustomInput";
import useCommon from "../store/CommonSlice";
import {Controller, useForm} from "react-hook-form";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {v4} from "uuid";
import {enqueueSnackbar} from "notistack";
import {$api, client} from "../utils/http";
import {components} from "../../schema";
import {useQueryClient} from "@tanstack/react-query";
import {APIError} from "../utils/ErrorDefinition";

type UserManagementPageProps = {

}


export const UserManagementPage: FC<UserManagementPageProps> = () => {
    const {t} = useTranslation()
    const queryClient = useQueryClient()
    const {data, error, isLoading} = $api.useQuery('get', '/api/v1/users/{username}', {
        params: {
            path: {
                username: 'me'
            }
        },
    })
    const updateProfile = $api.useMutation('put', '/api/v1/users/{username}')
    const {control, handleSubmit, setValue} = useForm<components["schemas"]["UserCoreUpdateModel"]>({
        defaultValues: {
            username: '',
            apiKey: ''
        }})


    useEffect(() => {
        if (data && data.username) {
            setValue('username', data.username)
        }
    }, [data])

    const update_settings = (data: components["schemas"]["UserCoreUpdateModel"]) => {
        if (data.password === '') {
            delete data.password
        }

        updateProfile.mutateAsync({
            body: data,
            params: {
                path: {
                    username: data!.username
                }
            },
        }).then(()=>{
            queryClient.setQueryData(['get', '/api/v1/users/{username}'], (oldData: any) => {
                return {
                    ...oldData,
                    ...data
                }
            })
        })
            .catch(e=>{
            if (e instanceof APIError) {
                enqueueSnackbar(t(e.details?.errorCode, e.details.arguments), {variant: 'error'})
            } else {
                enqueueSnackbar(e.message, {variant: 'error'})
            }
        })
    }

    if (isLoading  || !data) {
        return <div>{t('loading')}...</div>
    }

    return (
        <div className="md:w-3/6">
            <Heading1>{t('profile')}</Heading1>
            <div className="mt-5">
                <form onSubmit={handleSubmit(update_settings)}>
                    <div className="grid grid-cols-2 gap-5 mb-5">
                        <label className="ml-2 mt-2 text-sm text-(--fg-secondary-color)"
                               htmlFor="username">{t('username')}</label>
                        <Controller name="username" control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <CustomInput id="username" readOnly={data.readOnly} name={name}
                                                     onChange={onChange}
                                                     value={value}/>
                                    )}/>
                        <label className="ml-2 mt-2 text-sm text-(--fg-secondary-color)"
                               htmlFor="password">{t('password')}</label>
                        <Controller name="password" control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <CustomInput id="password" name={name}  readOnly={data.readOnly}
                                                     onChange={onChange}
                                                     value={value ?? ""}/>
                                    )}/>
                        <label className="ml-2 mt-2 text-sm text-(--fg-secondary-color)"
                               htmlFor="apiKey">{t('api-key')}</label>
                        <Controller name="apiKey" control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <div className="block relative">
                                            <CustomInput disabled={true} className="w-full" id="apiKey" name={name}
                                                     onChange={onChange} readOnly={data.readOnly}
                                                     value={value ?? ""}/>
                                            <button disabled={data.readOnly} type="button" className="material-symbols-outlined absolute right-2 top-1.5 text-(--fg-color)" onClick={()=>{
                                                setValue("apiKey", v4().replace(/-/g, ''))
                                            }}>cached</button>
                                        </div>
                                    )}/>
                    </div>

                    <CustomButtonPrimary disabled={data.readOnly} type="submit" className="float-right">{t('save')}</CustomButtonPrimary>
                </form>
            </div>
        </div>
    )
}
