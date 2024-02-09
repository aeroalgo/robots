using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F0 RID: 240
	[HandlerCategory("vvTrade"), HandlerName("Баров со входа в тек. позицию (от инструм.)")]
	public class BarsFromEntrySrc : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000742 RID: 1858 RVA: 0x00020680 File Offset: 0x0001E880
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition lastPositionActive = sec.get_Positions().GetLastPositionActive(barNum);
			if (lastPositionActive == null)
			{
				return 0.0;
			}
			return (double)(barNum - lastPositionActive.get_EntryBarNum());
		}
	}
}
