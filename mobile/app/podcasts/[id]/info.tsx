import {router, useLocalSearchParams} from "expo-router";
import {$api} from "@/client";
import {ScrollView, Text, View} from "react-native";
import {SafeAreaView} from "react-native-safe-area-context";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";
import {useStore} from "@/store/store";

export default function () {
    const { id } = useLocalSearchParams();
    const serverUrl = useStore((state) => state.serverUrl);
    const {data, isLoading, error} = $api.useQuery('get', '/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: id as string
            }
        }
    }, {
        enabled: !!serverUrl,
    })

    return <SafeAreaView>
        <MaterialIcons size={40} color="white" name="chevron-left" style={{
            position: 'absolute',
            top: 40,
            left: 20,
        }} onPress={()=>{
            router.back()
        }} />
        {
            !isLoading && data? <>
                <ScrollView overScrollMode="never" style={{
                    margin: 40,
                    marginTop: 60,
                    marginBottom: 80
                }}>
                    <Text style={{color: 'white', fontSize: 20}}>{data.summary}</Text>
                </ScrollView>
            </>: <></>
        }
    </SafeAreaView>
}
