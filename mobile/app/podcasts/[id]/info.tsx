import {useLocalSearchParams} from "expo-router";
import {$api} from "@/client";
import {ScrollView, Text, View} from "react-native";
import {SafeAreaView} from "react-native-safe-area-context";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";

export default function () {
    const { id } = useLocalSearchParams();
    const {data, isLoading, error} = $api.useQuery('get', '/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: id as string
            }
        }
    })

    return <SafeAreaView>
        <MaterialIcons size={40} color="white" name="chevron-left" style={{
            position: 'absolute',
            top: 40,
            left: 20,
        }} onPress={()=>{
            window.history.back()
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
