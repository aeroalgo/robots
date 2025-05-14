using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000010 RID: 16
	[HandlerCategory("vvIndicators"), HandlerName("Aroon")]
	public class Aroon : BasePeriodIndicatorHandler, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000086 RID: 134 RVA: 0x00003997 File Offset: 0x00001B97
		public IList<double> Execute(IList<double> src)
		{
			if (!this.AroonDown)
			{
				return this.ExecuteUp(src);
			}
			return this.ExecuteDown(src);
		}

		// Token: 0x06000085 RID: 133 RVA: 0x00003940 File Offset: 0x00001B40
		public IList<double> ExecuteDown(IList<double> src)
		{
			double[] array = new double[src.Count];
			for (int i = 1; i < src.Count; i++)
			{
				double num = vvSeries.LowestBarNum(src, i, base.get_Period());
				array[i] = 100.0 * ((double)base.get_Period() - num) / (double)base.get_Period();
			}
			return array;
		}

		// Token: 0x06000084 RID: 132 RVA: 0x000038E8 File Offset: 0x00001AE8
		public IList<double> ExecuteUp(IList<double> src)
		{
			double[] array = new double[src.Count];
			for (int i = 1; i < src.Count; i++)
			{
				double num = vvSeries.HighestBarNum(src, i, base.get_Period());
				array[i] = 100.0 * ((double)base.get_Period() - num) / (double)base.get_Period();
			}
			return array;
		}

		// Token: 0x1700002A RID: 42
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool AroonDown
		{
			// Token: 0x06000082 RID: 130 RVA: 0x000038D6 File Offset: 0x00001AD6
			get;
			// Token: 0x06000083 RID: 131 RVA: 0x000038DE File Offset: 0x00001ADE
			set;
		}

		// Token: 0x1700002B RID: 43
		public IContext Context
		{
			// Token: 0x06000087 RID: 135 RVA: 0x000039B0 File Offset: 0x00001BB0
			get;
			// Token: 0x06000088 RID: 136 RVA: 0x000039B8 File Offset: 0x00001BB8
			set;
		}
	}
}
