from abc import ABC, abstractmethod

class HDRBackend(ABC):
    @abstractmethod
    def check_dependencies(self):
        """Verifies that necessary tools (exe/binary) exist."""
        pass

    @abstractmethod
    def get_monitors(self):
        """Returns a list of monitor identifiers."""
        pass

    @abstractmethod
    def toggle_hdr(self, monitor_id, enable: bool):
        """Enables or disables HDR."""
        pass