import {
    AnyDhtHash, AnyLinkableHash,
    AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash'; // approx. 11kb
import {
    Zomelet,
}					from '@spartan-hc/zomelets'; // approx. 7kb
import {
    ScopedEntity,
}					from '@spartan-hc/entities';
import {
    intoStruct,
    OptionType, VecType,
}					from '@whi/into-struct';


//
// Types & Structs
//
export const ContentStruct = {
    "text":			String,
    "author":			AgentPubKey,
    "group_ref":		{
	"id": ActionHash,
	"rev": ActionHash,
    },

    "published_at":		Number,
    "last_updated":		Number,
};

export function ContentEntry ( entry ) {
    return intoStruct( entry, ContentStruct );
}

export class Content extends ScopedEntity {
    static STRUCT		= ContentStruct;
}



//
// Zomelets
//
export const BasicUsageZomelet	        = new Zomelet({
    //
    // Content
    //
    async create_content ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },
    async get_content ( input ) {
	const result			= await this.call( input );

	return ContentEntry( result );
    },
    async update_content ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },
    async get_group_content ( input ) {
	const result			= await this.call( input );

	return result.map( ([[origin_addr, latest_addr], data]) => {
            return [
                [
                    new ActionHash(origin_addr),
                    new ActionHash(latest_addr),
                ],
                ContentEntry( data ),
            ]
        });
    },
});


export default {
    BasicUsageZomelet,
}
