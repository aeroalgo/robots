using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000148 RID: 328
	[HandlerCategory("vvMACD"), HandlerName("MACDadv")]
	public class MACDadv : MACDBase, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000A1A RID: 2586 RVA: 0x0002A2F4 File Offset: 0x000284F4
		public IList<double> Execute(IList<double> src)
		{
			IList<double> list = base.CalcMACD(src, this.FastPeriod, this.SlowPeriod);
			IList<double> result = Series.EMA(list, this.SignalLinePeriod);
			if (!this.DrawSignalLine)
			{
				return list;
			}
			return result;
		}

		// Token: 0x1700034F RID: 847
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool DrawSignalLine
		{
			// Token: 0x06000A18 RID: 2584 RVA: 0x0002A2E2 File Offset: 0x000284E2
			get;
			// Token: 0x06000A19 RID: 2585 RVA: 0x0002A2EA File Offset: 0x000284EA
			set;
		}

		// Token: 0x1700034D RID: 845
		[HandlerParameter(true, "12", Min = "5", Max = "40", Step = "1")]
		public int FastPeriod
		{
			// Token: 0x06000A14 RID: 2580 RVA: 0x0002A2C0 File Offset: 0x000284C0
			get;
			// Token: 0x06000A15 RID: 2581 RVA: 0x0002A2C8 File Offset: 0x000284C8
			set;
		}

		// Token: 0x1700034E RID: 846
		[HandlerParameter(true, "9", Min = "3", Max = "20", Step = "1")]
		public int SignalLinePeriod
		{
			// Token: 0x06000A16 RID: 2582 RVA: 0x0002A2D1 File Offset: 0x000284D1
			get;
			// Token: 0x06000A17 RID: 2583 RVA: 0x0002A2D9 File Offset: 0x000284D9
			set;
		}

		// Token: 0x1700034C RID: 844
		[HandlerParameter(true, "26", Min = "10", Max = "40", Step = "1")]
		public int SlowPeriod
		{
			// Token: 0x06000A12 RID: 2578 RVA: 0x0002A2AF File Offset: 0x000284AF
			get;
			// Token: 0x06000A13 RID: 2579 RVA: 0x0002A2B7 File Offset: 0x000284B7
			set;
		}
	}
}
