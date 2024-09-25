import {
    AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash'; // approx. 11kb
import {
    ScopedEntity,
}					from '@spartan-hc/entities';
import {
    intoStruct,
    OptionType, VecType,
}					from '@whi/into-struct';


export const GroupStruct = {
    "admins":			VecType( AgentPubKey ),
    "members":			VecType( AgentPubKey ),

    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			{},
};

export function GroupEntry ( entry ) {
    return intoStruct( entry, GroupStruct );
}

export class Group extends ScopedEntity {
    static STRUCT		= GroupStruct;
}

export default {
    GroupStruct,
    GroupEntry,
    Group,
};
