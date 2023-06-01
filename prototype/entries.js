
import { faker }			from '@faker-js/faker';
import {
    HoloHash, AgentPubKey,
    ActionHash, EntryHash,
    AnyLinkableHash,
}					from '@whi/holo-hash';
import {
    AppEntryType,
    OptionType, VecType, MapType,
    PathEntry,
    BoilerPlateSet,
}					from '@whi/holochain-prototyping';


export class GroupEntry extends AppEntryType {
    static struct			= {
	"admins":			VecType( AgentPubKey ),
	"members":			VecType( AgentPubKey ),

	"published_at":			Date,
	"last_updated":			Date,

	"metadata":			Object,
    };

    get authorities () {
	return [ ...this.admins, ...this.members ];
    }

    adminsSet () {
	return new BoilerPlateSet( this.admins );
    }

    membersSet () {
	return new BoilerPlateSet( this.members );
    }

    authoritiesSet () {
	return new BoilerPlateSet( this.authorities );
    }
}


export class SubjectEntry extends AppEntryType {
    static struct			= {
	"author":			[
	    ActionHash, // GroupEntry Create
	    ActionHash, // GroupEntry Revision
	],
	"message":			String,

	"published_at":			Date,
	"last_updated":			Date,

	"metadata":			Object,
    };
}


export { PathEntry }			from '@whi/holochain-prototyping';

export class GroupMemberAnchorEntry extends PathEntry {
    static struct			= [
	ActionHash,
	AgentPubKey,
    ];

    // toJSON () {
    // 	return [
    // 	    this[0].toString(true),
    // 	    this[1].toString(true),
    // 	];
    // }
}

export class GroupMemberArchiveAnchorEntry extends PathEntry {
    static struct			= [
	ActionHash,
	AgentPubKey,
	"archive",
    ];

    // toJSON () {
    // 	return [
    // 	    this[0].toString(true),
    // 	    this[1].toString(true),
    // 	    this[2],
    // 	];
    // }
}


export function groupInput ( admin, ...members ) {
    return {
	"admins":		[ admin ],
	"members":		members,

	"published_at":		new Date(),
	"last_updated":		new Date(),

	"metadata":		{},
    };
}


export function subjectInput ( group_id, group_revision ) {
    return {
	"author":		[ group_id, group_revision || group_id ],
	"message":		faker.lorem.sentence(),

	"published_at":		new Date(),
	"last_updated":		new Date(),

	"metadata":		{},
    };
}



export default {
    GroupEntry,
    SubjectEntry,
    PathEntry,
    GroupMemberAnchorEntry,
    GroupMemberArchiveAnchorEntry,

    groupInput,
    subjectInput,
};
