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

    get contributors () {
        return [
            ...this.admins,
            ...this.members,
        ];
    }

    isContributor ( agent_pubkey ) {
        return this.contributors
            .map( agent => String(agent) )
            .includes( String(new AgentPubKey(agent_pubkey)) );
    }

    isAdmin ( agent_pubkey ) {
        return this.admins
            .map( agent => String(agent) )
            .includes( String(new AgentPubKey(agent_pubkey)) );
    }

    isMember ( agent_pubkey ) {
        return this.members
            .map( agent => String(agent) )
            .includes( String(new AgentPubKey(agent_pubkey)) );
    }
}

export default {
    GroupStruct,
    GroupEntry,
    Group,
};
