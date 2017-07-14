initSidebarItems({"constant":[["APPLICATION",""],["DIRECT_SAMPLE",""],["DUPLEX",""],["HARDWARE",""],["MIDI_GENERIC",""],["MIDI_GM",""],["MIDI_GM2",""],["MIDI_GS",""],["MIDI_MT32",""],["MIDI_XG",""],["NO_EXPORT",""],["PORT",""],["READ",""],["SAMPLE",""],["SOFTWARE",""],["SPECIFIC",""],["SUBS_READ",""],["SUBS_WRITE",""],["SYNC_READ",""],["SYNC_WRITE",""],["SYNTH",""],["SYNTHESIZER",""],["WRITE",""]],"enum":[["EventType","SND_SEQ_EVENT_xxx constants"]],"struct":[["Addr","snd_seq_addr_t wrapper"],["ClientInfo","snd_seq_client_info_t wrapper"],["ClientIter","Iterates over clients connected to the seq API (both kernel and userspace clients)."],["Connect","snd_seq_connect_t wrapper"],["EvCtrl",""],["EvNote",""],["EvQueueControl","snd_seq_ev_queue_control_t wrapper"],["EvResult","snd_seq_result_t wrapper"],["Event","snd_seq_event_t wrapper"],["Input","Struct for receiving input events from a sequencer. The methods offered by this object may modify the internal input buffer of the sequencer, which must not happen while an `Event` is alive that has been obtained from a call to `event_input` (which takes `Input` by mutable reference for this reason). This is because the event might directly reference the sequencer's input buffer for variable-length messages (e.g. Sysex)."],["MidiEvent","snd_midi_event_t Wrapper"],["PortCap","[SND_SEQ_PORT_CAP_xxx]http://www.alsa-project.org/alsa-doc/alsa-lib/group___seq_port.html) constants "],["PortInfo","snd_seq_port_info_t wrapper"],["PortIter","Iterates over clients connected to the seq API (both kernel and userspace clients)."],["PortSubscribe","snd_seq_port_subscribe_t wrapper"],["PortType","[SND_SEQ_PORT_TYPE_xxx]http://www.alsa-project.org/alsa-doc/alsa-lib/group___seq_port.html) constants "],["QueueTempo","snd_seq_queue_tempo_t wrapper"],["Seq","snd_seq_t wrapper"]],"trait":[["EventData","Low level methods to set/get data on an Event. Don't use these directly, use generic methods on Event instead."]]});