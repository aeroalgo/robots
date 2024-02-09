using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A4 RID: 420
	[HandlerCategory("vvAverages"), HandlerName("vaMA")]
	public class vaMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D4F RID: 3407 RVA: 0x0003AAA4 File Offset: 0x00038CA4
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("vaMA", new string[]
			{
				this.Length.ToString(),
				this.doublesmooth.ToString(),
				src.GetHashCode().ToString()
			}, () => vaMA.GenVaMA(this.Context, src, this.Length, this.doublesmooth));
		}

		// Token: 0x06000D4E RID: 3406 RVA: 0x0003A8BC File Offset: 0x00038ABC
		public static IList<double> GenVaMA(IContext ctx, IList<double> src, int _vamalength, bool _doublesmooth)
		{
			int count = src.Count;
			double[] array = new double[count];
			_vamalength = Math.Max(1, _vamalength);
			IList<double> data = ctx.GetData("ema", new string[]
			{
				_vamalength.ToString(),
				src.GetHashCode().ToString()
			}, () => EMA.EMA_MT(src, (double)_vamalength));
			for (int i = 0; i < count; i++)
			{
				if (i >= _vamalength)
				{
					double num = data[i] - data[i - _vamalength / 4];
					double num2 = data[i] - 2.0 * data[i - _vamalength / 4] + data[i - _vamalength / 8];
					double num3 = data[i] - 3.0 * data[i - _vamalength / 4] + 3.0 * data[i - _vamalength / 8] - data[i - _vamalength / 12];
					array[i] = data[i] + num + num2 / 2.0 + num3 / 6.0;
				}
				else
				{
					array[i] = src[i];
				}
			}
			IList<double> result = array;
			if (_doublesmooth)
			{
				result = EMA.EMA_MT(array, (double)(_vamalength / 4));
			}
			return result;
		}

		// Token: 0x17000453 RID: 1107
		public IContext Context
		{
			// Token: 0x06000D50 RID: 3408 RVA: 0x0003AB22 File Offset: 0x00038D22
			get;
			// Token: 0x06000D51 RID: 3409 RVA: 0x0003AB2A File Offset: 0x00038D2A
			set;
		}

		// Token: 0x17000452 RID: 1106
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool doublesmooth
		{
			// Token: 0x06000D4C RID: 3404 RVA: 0x0003A88E File Offset: 0x00038A8E
			get;
			// Token: 0x06000D4D RID: 3405 RVA: 0x0003A896 File Offset: 0x00038A96
			set;
		}

		// Token: 0x17000451 RID: 1105
		[HandlerParameter(true, "14", Min = "0", Max = "30", Step = "1")]
		public int Length
		{
			// Token: 0x06000D4A RID: 3402 RVA: 0x0003A87D File Offset: 0x00038A7D
			get;
			// Token: 0x06000D4B RID: 3403 RVA: 0x0003A885 File Offset: 0x00038A85
			set;
		}
	}
}
