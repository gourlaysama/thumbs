# Copyright (C) 2022 Antoine Gourlay <antoine@gourlay.fr>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import os.path

from gi import require_version
require_version('Nautilus', '3.0')
from gi.repository import Nautilus, GObject, Gio, GLib


class DeleteThumbnailsAction(GObject.GObject, Nautilus.MenuProvider):
    def _process_files(self, files):
        paths = []
        for file in files:
            path = file.get_location().get_path()
            if path and path not in paths:
                paths.append(path)
        return paths

    def _make_menu(self, name, paths):
        menu = Nautilus.MenuItem(name=name, label='Delete thumbnails', icon='edit-delete-symbolic')
        menu.connect('activate', self._run_thumbs, paths)
        return menu

    def get_file_items(self, window, files):
        paths = self._process_files(files)
        if paths:
            return [self._make_menu(name='ThumbsNautilus::delete_thumbs_for_files', paths=paths)]
        else:
            return []

    def get_background_items(self, window, file):
        paths = self._process_files([file])
        if paths:
            return [self._make_menu(name='ThumbsmNautilus::delete_thumbs_for_folder', paths=paths)]
        else:
            return []

    def _run_thumbs(self, _menu, paths):
        cmd = ['thumbs', 'delete', '-r', '-f'] + paths
        Gio.Subprocess.new(cmd, Gio.SubprocessFlags.NONE)

