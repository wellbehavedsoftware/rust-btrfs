//! This module contains an interface to the kernel's space info facility.
//!
//! Please note, that this information is not representetive of the actual
//! storage used, and so is only useful in specific cases. The "file system
//! info" functionality is, in many cases, more likely to be useful.

use linux::imports::*;

// ---------- get space info

pub fn get_space_info (
	file_descriptor: libc::c_int,
) -> Result <Vec <SpaceInfo>, String> {

	// open directory

	let mut num_spaces = 0;
	let mut c_space_info;

	loop {

		c_space_info =
			try! (
				get_c_space_info (
					file_descriptor,
					num_spaces));

		if c_space_info.args.total_spaces
			<= c_space_info.args.space_slots {

			break;

		}

		num_spaces =
			c_space_info.args.total_spaces;

	}

	// create return value

	let mut space_infos: Vec <SpaceInfo> =
		vec! [];

	for c_space_info in c_space_info.infos.iter () {

		space_infos.push (
			SpaceInfo {

			group_type:
				GroupType::from (
					c_space_info.flags),

			group_profile:
				GroupProfile::from (
					c_space_info.flags),

			used_bytes:
				c_space_info.used_bytes,

			total_bytes:
				c_space_info.total_bytes,

			}
		);

	}

	Ok (space_infos)

}

// low level wrapper

struct CSpaceInfoResult {
	args: IoctlSpaceArgs,
	infos: Vec <IoctlSpaceInfo>,
}

fn get_c_space_info (
	file_descriptor: libc::c_int,
	num_spaces: u64,
) -> Result <CSpaceInfoResult, String> {

	// allocate buffer

	let c_space_buffer_size =
		mem::size_of::<IoctlSpaceArgs> ()
		+ num_spaces as usize
			* mem::size_of::<IoctlSpaceInfo> ();

	let mut c_space_buffer: Vec <u8> =
		Vec::from_iter (
			iter::repeat (0u8).take (
				c_space_buffer_size));

	let (c_space_args_buffer, c_space_infos_buffer) =
		c_space_buffer.split_at_mut (
			mem::size_of::<IoctlSpaceArgs> ());

	// split buffer

	let c_space_args_slice: & mut [IoctlSpaceArgs] =
		unsafe {
			slice::from_raw_parts_mut (
				mem::transmute (
					c_space_args_buffer.as_mut_ptr ()),
				1)
		};

	let c_space_args =
		& mut c_space_args_slice [0];

	let c_space_infos: & mut [IoctlSpaceInfo] =
		unsafe {
			slice::from_raw_parts_mut (
				mem::transmute (
					c_space_infos_buffer.as_mut_ptr ()),
				num_spaces as usize)
		};

	// get info

	c_space_args.space_slots =
		num_spaces;

	let get_space_args_real_result =
		unsafe {
			ioctl_space_info (
				file_descriptor,
				c_space_args as * mut IoctlSpaceArgs)
		};

	if let Err(e) = get_space_args_real_result {

		return Err (
			"Error getting btrfs space information".to_string ()
		);

	}

	// return

	Ok (
		CSpaceInfoResult {
			args: * c_space_args,
			infos: c_space_infos.to_vec (),
		}
	)

}

// ex: noet ts=4 filetype=rust
