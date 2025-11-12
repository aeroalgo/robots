using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200005C RID: 92
	[HandlerCategory("vvIndicators"), HandlerName("StDev normalized")]
	public class StDevNrm : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600034D RID: 845 RVA: 0x00012F90 File Offset: 0x00011190
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("StDevNormalized", new string[]
			{
				this.StdPeriod.ToString(),
				this.NormalizationPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => this.GenStDevNrm(src, this.StdPeriod, this.NormalizationPeriod, this.Context));
		}

		// Token: 0x0600034C RID: 844 RVA: 0x00012DB0 File Offset: 0x00010FB0
		public IList<double> GenStDevNrm(IList<double> src, int _StdPeriod, int _NormPeriod, IContext context)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> stdev = context.GetData("stdev", new string[]
			{
				_StdPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.StDev(src, _StdPeriod));
			IList<double> data = context.GetData("hhv", new string[]
			{
				_NormPeriod.ToString(),
				stdev.GetHashCode().ToString()
			}, () => Series.Highest(stdev, _NormPeriod));
			IList<double> data2 = context.GetData("llv", new string[]
			{
				_NormPeriod.ToString(),
				stdev.GetHashCode().ToString()
			}, () => Series.Lowest(stdev, _NormPeriod));
			for (int i = 0; i < count; i++)
			{
				double num = data2[i];
				double num2 = data[i];
				if (num != num2)
				{
					array[i] = 100.0 * (stdev[i] - num) / (num2 - num);
				}
				else
				{
					array[i] = 0.5;
				}
			}
			return array;
		}

		// Token: 0x1700011C RID: 284
		public IContext Context
		{
			// Token: 0x0600034E RID: 846 RVA: 0x0001300E File Offset: 0x0001120E
			get;
			// Token: 0x0600034F RID: 847 RVA: 0x00013016 File Offset: 0x00011216
			set;
		}

		// Token: 0x1700011B RID: 283
		[HandlerParameter(true, "10", Min = "1", Max = "30", Step = "1")]
		public int NormalizationPeriod
		{
			// Token: 0x0600034A RID: 842 RVA: 0x00012D5C File Offset: 0x00010F5C
			get;
			// Token: 0x0600034B RID: 843 RVA: 0x00012D64 File Offset: 0x00010f32
			set;
		}

		// Token: 0x1700011A RID: 282
		[HandlerParameter(true, "10", Min = "1", Max = "30", Step = "1")]
		public int StdPeriod
		{
			// Token: 0x06000348 RID: 840 RVA: 0x00012D4B File Offset: 0x00010F4B
			get;
			// Token: 0x06000349 RID: 841 RVA: 0x00012D53 File Offset: 0x00010F53
			set;
		}
	}
}
