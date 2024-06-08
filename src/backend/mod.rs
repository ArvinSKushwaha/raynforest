//! This module abstracts over the interface provided by wgpu to be more me- friendly.
//! Also, kudos to [Ben Hansen](https://github.com/sotrh/) for the amazing resources
//! over on their GitHub and website! A lot of techniques used in this module are
//! heavily inspired by their repo [sotrh/learn-wgpu](https://github.com/sotrh/learn-wgpu).

mod buffers;
mod device;
mod pipeline;
mod traits;
mod util;
