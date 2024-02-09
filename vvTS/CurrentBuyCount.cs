using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Realtime;

namespace vvTSLtools
{
	// Token: 0x020000C6 RID: 198
	[HandlerCategory("vvTrade"), HandlerName("Сейчас заявок на покупку")]
	public class CurrentBuyCount : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006D9 RID: 1753 RVA: 0x0001EAFC File Offset: 0x0001CCFC
		public IList<double> Execute(ISecurity sec)
		{
			ISecurityRt securityRt = sec as ISecurityRt;
			double num = (securityRt == null) ? 0.0 : (securityRt.get_FinInfo().get_BuyCount().HasValue ? securityRt.get_FinInfo().get_BuyCount().Value : 0.0);
			double[] array = new double[sec.get_Bars().Count];
			for (int i = 0; i < array.Length; i++)
			{
				array[i] = num;
			}
			return array;
		}
	}
}
