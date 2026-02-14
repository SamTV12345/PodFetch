import {ThemedText} from "@/components/ThemedText";

export default function ({ children }: { children: string }) {
    return (
        <ThemedText style={{color: 'white', fontSize: 24, fontWeight: 'bold', paddingLeft: 10, paddingBottom: 5}}>{children}</ThemedText>
    )
}
