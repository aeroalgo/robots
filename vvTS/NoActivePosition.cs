using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F7 RID: 247
	[HandlerCategory("vvTrade"), HandlerName("Нет активной позиции")]
	public class NoActivePosition : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000754 RID: 1876 RVA: 0x0002089C File Offset: 0x0001EA9C
		public bool Execute(ISecurity sec, int barNum)
		{
			int activePositionCount = sec.get_Positions().get_ActivePositionCount();
			return activePositionCount == 0;
		}
	}
}
