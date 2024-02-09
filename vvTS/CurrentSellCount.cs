using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Realtime;

namespace vvTSLtools
{
	// Token: 0x020000C7 RID: 199
	[HandlerCategory("vvTrade"), HandlerName("Сейчас заявок на продажу")]
	public class CurrentSellCount : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006DB RID: 1755 RVA: 0x0001EB80 File Offset: 0x0001CD80
		public IList<double> Execute(ISecurity sec)
		{
			ISecurityRt securityRt = sec as ISecurityRt;
			double num = (securityRt == null) ? 0.0 : (securityRt.get_FinInfo().get_SellCount().HasValue ? securityRt.get_FinInfo().get_SellCount().Value : 0.0);
			double[] array = new double[sec.get_Bars().Count];
			for (int i = 0; i < array.Length; i++)
			{
				array[i] = num;
			}
			return array;
		}
	}
}
