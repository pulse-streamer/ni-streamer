from nistreamer_backend import StreamerWrap
from .card import BaseCardProxy, AOCardProxy, DOCardProxy
from typing import Optional, Literal, Union, Tuple, Type


class NIStreamer:

    def __init__(self):
        self._streamer = StreamerWrap()
        self._ao_cards = dict()
        self._do_cards = dict()

    def __getitem__(self, item):
        if item in self._ao_cards.keys():
            return self._ao_cards[item]
        elif item in self._do_cards.keys():
            return self._do_cards[item]
        else:
            raise KeyError(f'There is no card with max_name "{item}" registered')

    def __repr__(self):
        return (
            f'NIStreamer instance\n'
            f'\n'
            f'AO cards: {list(self._ao_cards.keys())}\n'
            f'DO cards: {list(self._do_cards.keys())}\n'
            f'\n'
            f'Hardware settings:\n'
            f'\t10MHz ref provider: {self.ref_clk_provider}\n'
            f'\t  Starts-last card: {self.starts_last}'
        )

        # # FixMe: TypeError: Object of type AOCard is not JSON serializable
        # return (
        #     f'Experiment class.\n'
        #     f'The following AO cards have been added already:\n'
        #     f'{json.dumps(self._ao_card_dict, indent=4)}\n'
        #     f'\n'
        #     f'The following DO cards have been added already:\n'
        #     f'{json.dumps(self._do_card_dict, indent=4)}'
        # )

    def _add_card(
            self,
            card_type: Literal['AO', 'DO'],
            max_name: str,
            samp_rate: float,
            proxy_class: Type[BaseCardProxy],
            nickname: Optional[str] = None,
     ) -> BaseCardProxy:

        if card_type == 'AO':
            streamer_method = StreamerWrap.add_ao_dev
            target_dict = self._ao_cards
        elif card_type == 'DO':
            streamer_method = StreamerWrap.add_do_dev
            target_dict = self._do_cards
        else:
            raise ValueError(f'Invalid card type "{card_type}". Valid type strings are "AO" and "DO"')

        # Call to NIStreamer struct in the compiled Rust backend
        streamer_method(
            self._streamer,
            name=max_name,
            samp_rate=samp_rate
        )
        # Proxy object
        proxy = proxy_class(
            _streamer=self._streamer,
            max_name=max_name,
            nickname=nickname
        )
        target_dict[max_name] = proxy
        return proxy

    def add_ao_card(
            self,
            max_name: str,
            samp_rate: float,
            nickname: Optional[str] = None,
            proxy_class: Optional[Type[BaseCardProxy]] = AOCardProxy
    ):
        """Add a digital output card to NIStreamer

        :param max_name: name of the card as shown in NI MAX
        :param samp_rate: sample rate in Hz
        :param nickname: (optional) human-readable name (e.g. "Fast AO card"; used for pulse sequence visualization)
        :param proxy_class: (optional) custom subclass of `BaseDevProxy` to use for the device proxy
        :return: proxy_class instance
        """
        return self._add_card(
            card_type='AO',
            max_name=max_name,
            samp_rate=samp_rate,
            nickname=nickname,
            proxy_class=proxy_class
        )

    def add_do_card(
            self,
            max_name: str,
            samp_rate: float,
            nickname: Optional[str] = None,
            proxy_class: Optional[Type[BaseCardProxy]] = DOCardProxy
    ):
        """Add an analog output card to NIStreamer

        :param max_name: name of the card as shown in NI MAX
        :param samp_rate: sample rate in Hz
        :param nickname: (optional) human-readable name (e.g. "Main DO card"; used for pulse sequence visualization)
        :param proxy_class: (optional) custom subclass of `BaseDevProxy` to use for the device proxy
        :return: proxy_class instance
        """
        return self._add_card(
            card_type='DO',
            max_name=max_name,
            samp_rate=samp_rate,
            nickname=nickname,
            proxy_class=proxy_class
        )

    @property
    def starts_last(self) -> Union[str, None]:
        """Specifies which card starts last. Typically, this is needed when start trigger or shared sample clock are used
        for hardware synchronisation.

        Format:
            * `dev_name: str` - this card will wait for all other ones to start first;
            * `None` - each threads will start its task whenever it is ready without waiting for anyone.

        Specifically, it determines which thread waits to call `ni_task.start()` until all other threads have called
        their `ni_task.start()` first.

        If a given card is awaiting a start trigger / uses external sample clock,
        it should start before the card which produces this signal (the 'primary card'),
        otherwise the trigger pulse / first few clock pulses can be missed.

        Streamer provides option to specify a single primary card and handles the necessary thread sync
        to make sure the designated card calls `ni_task.start()` last.
        """
        return self._streamer.get_starts_last()

    @starts_last.setter
    def starts_last(self, name: Union[str, None]):
        self._streamer.set_starts_last(name=name)

    @property
    def ref_clk_provider(self) -> Union[Tuple[str, str], None]:
        """Specifies which card exports its 10MHz reference signal for use by all other cards.

        Format:
            * `(card_name: str, term_name: str)` - card `card_name` exports 10MHz ref to terminal `term_name`
            * `None` - no card exports

        Technical details:
            (1) NIStreamer uses 'run-based' static reference clock export: signal is exported during `cfg_run()`
            and un-exported during `close_run()` calls. This export is not dependent on any NI tasks.

            As a result, the provider can be any card supporting 10MHz ref export.
            It does not have to get any instructions or even be registered in the `NIStreamer`.

            (2) Users can manually do static export of 10MHz ref from any card by calling `utils.share_10mhz_ref()`.
            However, such export will not be automatically undone and the user has to manually call
            either `utils.unshare_10mhz_ref()` or `utils.reset_dev()`.

            This is dangerous since forgetting to un-export can easily lead to foot guns. Only do this if you
            need to go beyond standard configuration. For most cases just specify `ref_clk_provider` since it does
            precisely that but also does everything possible to automatically undo the export whenever the run stops.
        """
        return self._streamer.get_ref_clk_provider()

    @ref_clk_provider.setter
    def ref_clk_provider(self, dev_and_term: Union[Tuple[str, str], None]):
        self._streamer.set_ref_clk_provider(provider=dev_and_term)

    def got_instructions(self) -> bool:
        return self._streamer.got_instructions()

    def last_instr_end_time(self) -> Union[float, None]:
        return self._streamer.last_instr_end_time()

    def compile(self, stop_time: Optional[float] = None) -> float:
        return self._streamer.compile(stop_time=stop_time)

    def validate_compile_cache(self):
        """Returns None if compile cache is valid (up-to-date with the current edit cache).
        Raises ValueError if compile cache is invalid (typical reason - users forgot to re-compile
        after adding more instructions).

        :return: None if compile cache is valid (up-to-date with the current edit cache)
        """
        self._streamer.validate_compile_cache()

    def run(self, nreps: Optional[int] = 1, bufsize_ms: Optional[float] = 150) -> None:
        try:
            self._streamer.cfg_run(bufsize_ms=bufsize_ms)
            for i in range(nreps):
                self._streamer.stream_run(calc_next=(i < nreps - 1))
        finally:
            self._streamer.close_run()

    def add_reset_instr(self, reset_time: Optional[float] = None):
        self._streamer.add_reset_instr(reset_time=reset_time)

    def clear_edit_cache(self):
        self._streamer.clear_edit_cache()

    def reset_all(self):
        self._streamer.reset_all()
