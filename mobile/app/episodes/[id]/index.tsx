import {SafeAreaView} from "react-native-safe-area-context";
import {ScrollView, Text} from "react-native";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";
import {router} from "expo-router";
import Heading2 from "@/components/text/Heading2";
import {$api} from "@/client";
import {useStore} from "@/store/store";

export default function () {
    const serverUrl = useStore((state) => state.serverUrl);
    const {} = $api.useQuery("get", "/api/v1/info", {}, {
        enabled: !!serverUrl,
    })
    const podcastDetailedData = {
        data: {
            name: "test"
        }
    }

    return <SafeAreaView>
        <ScrollView overScrollMode="never">
            <MaterialIcons size={40} color="white" name="chevron-left" style={{
                position: 'absolute',
                top: 20,
                left: 20,
            }} onPress={()=>{
                router.back()
            }} />
            <Text style={{color: 'white'}}>test</Text>
        </ScrollView>
        <Heading2 styles={{marginRight: 'auto', marginLeft: 'auto', width: '95%', marginTop: 10, paddingBottom: 0}}>{podcastDetailedData.data.name}</Heading2>
    </SafeAreaView>
}
