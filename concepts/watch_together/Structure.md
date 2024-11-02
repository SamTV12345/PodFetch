# WatchTogetherUser

WatchTogetherUser represents the user registered in a watch together room.
Subject is the id of the user which can be used to identity the user in the database.
name is the name of the user. It can change over time.
user_id is the back reference to the PodFetch user object. This field can be null as a user can be a guest user who joined by the room id. 

# WatchTogether

This is the actual room where the users can watch together.
id is the unique identifier of the room. Once saved this is always present.
room_id is the room key of the room. Think of it as the room code in Kahoot.
A room id is generated when a user creates a room. But he/she can also recalculate the room id.
A room name is just the display name like "Movie Night" or "Game of Thrones". This can be changed by the user.


# WatchTogetherUsersToRoomMapping
Identifies the connection between the user and the room.
It contains the user_id and the room_id. This is a many to many relationship as a user can be in multiple rooms and a room can have multiple users.
Additionally, there is the status field which contains the status and or role of the user.

Pending means an admin has not yet approved the user to join the room.
Accepted means the user is in the room.
Rejected means the user is not allowed to join the room.
Admin means the user is an admin of the room.
User means the user is a normal user of the room.